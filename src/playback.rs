//! Reusable playback orchestration contract for the native desktop prototype.
//! The backend supplies entitlement checks and media URLs. Platform adapters
//! ultimately own rendering, decoding, DRM and native transport controls.

use serde_json::Value;
use std::fmt;
use std::io::Read;
use std::process::{Command, Stdio};
use std::time::Duration;

const MAX_RESPONSE: u64 = 64 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaybackSettings {
    pub api_base: String,
    pub locale: String,
    /// Local development only. This is never sent to non-loopback hosts.
    pub demo_tier: Option<String>,
}

impl PlaybackSettings {
    pub fn from_project(json: &str, locale_override: Option<&str>, demo_tier: Option<&str>)
        -> Result<Self, String>
    {
        let project: Value = serde_json::from_str(json)
            .map_err(|e| format!("invalid project JSON: {e}"))?;
        let runtime = project.get("runtime").ok_or("missing runtime configuration")?;
        let base = runtime.get("api").and_then(|v| v.get("baseUrl"))
            .and_then(Value::as_str).ok_or("runtime.api.baseUrl is missing")?;
        let base = base.trim_end_matches('/');
        let parsed: ureq::http::Uri = base.parse()
            .map_err(|e| format!("invalid API endpoint: {e}"))?;
        if parsed.scheme_str() != Some("https") && parsed.scheme_str() != Some("http") {
            return Err("playback API must use HTTP(S)".into());
        }
        if parsed.host().is_none() {
            return Err("playback API hostname missing".into());
        }
        let locale = locale_override
            .or_else(|| runtime.get("defaultLocale").and_then(Value::as_str))
            .unwrap_or("en-AU");
        if !valid_locale(locale) { return Err("invalid playback locale".into()); }
        let demo_tier = demo_tier
            .filter(|tier| ["free", "explorer", "infinite"].contains(tier))
            .map(str::to_owned);
        Ok(Self {api_base: base.to_owned(), locale: locale.to_owned(), demo_tier})
    }

    fn is_loopback(&self) -> bool {
        self.api_base.parse::<ureq::http::Uri>().ok()
            .and_then(|uri| uri.host().map(|host| host=="localhost"||host=="127.0.0.1"))
            .unwrap_or(false)
    }

    fn playback_endpoint(&self, id: &str) -> Result<String, PlaybackError> {
        if id.is_empty() || id.len() > 200 ||
            !id.chars().all(|ch| ch.is_ascii_alphanumeric() || ch=='-' || ch=='_') {
            return Err(PlaybackError::new("STVK.PLAYBACK.BAD_CONTENT_ID",
                "Invalid media identifier"));
        }
        Ok(format!("{}/playback/{}?locale={}", self.api_base, id, self.locale))
    }
}

fn valid_locale(locale: &str) -> bool {
    !locale.is_empty() && locale.len() <= 35 &&
        locale.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaybackError {
    pub code: String,
    pub message: String,
}
impl PlaybackError {
    fn new(code: &str, message: impl Into<String>) -> Self {
        Self { code: code.into(), message: message.into() }
    }
}
impl fmt::Display for PlaybackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} — {}", self.code, self.message)
    }
}
impl std::error::Error for PlaybackError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaSession {
    pub content_id: String,
    pub title: String,
    pub url: String,
    pub is_live: bool,
    pub simulated_live: bool,
}

/// Resolve the source through the backend. Never construct Cloudflare Stream
/// URLs or bypass entitlement checks in the client.
pub fn resolve(settings: &PlaybackSettings, id: &str) -> Result<MediaSession, PlaybackError> {
    let url = settings.playback_endpoint(id)?;
    let agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(8)))
        .http_status_as_error(false)
        .build()
        .new_agent();
    let mut req = agent.get(&url).header("Accept", "application/json");
    if settings.is_loopback() {
        if let Some(ref tier) = settings.demo_tier {
            req = req.header("x-demo-user", &format!("demo-{tier}"));
        }
    }
    let mut response = req.call().map_err(|e| PlaybackError::new(
        "STVK.PLAYBACK.NETWORK_ERROR", format!("Playback API unavailable: {e}")
    ))?;
    let status = response.status().as_u16();
    let mut body = String::new();
    response.body_mut().as_reader().take(MAX_RESPONSE + 1)
        .read_to_string(&mut body)
        .map_err(|e| PlaybackError::new("STVK.PLAYBACK.INVALID_RESPONSE", e.to_string()))?;
    if body.len() as u64 > MAX_RESPONSE {
        return Err(PlaybackError::new("STVK.PLAYBACK.INVALID_RESPONSE",
            "Playback response exceeds 64 KiB"));
    }
    let data: Value = serde_json::from_str(&body).map_err(|e| PlaybackError::new(
        "STVK.PLAYBACK.INVALID_RESPONSE", format!("Malformed JSON: {e}")
    ))?;
    if status >= 400 {
        let code = data.pointer("/error/code").and_then(Value::as_str)
            .unwrap_or("STVK.PLAYBACK.HTTP_ERROR");
        let message = data.pointer("/error/message").and_then(Value::as_str)
            .unwrap_or("Playback request was rejected");
        return Err(PlaybackError::new(code, message));
    }
    if status != 200 || data.get("entitled").and_then(Value::as_bool) != Some(true) {
        return Err(PlaybackError::new("STVK.PLAYBACK.INVALID_RESPONSE",
            "Playback API did not authorise a source"));
    }
    let source = data.get("source").ok_or_else(|| PlaybackError::new(
        "STVK.PLAYBACK.INVALID_RESPONSE", "Missing playback source"
    ))?;
    let selected = ["progressive","hls","dash"].iter()
        .find_map(|key| source.get(*key).and_then(Value::as_str).filter(|v| !v.is_empty()))
        .ok_or_else(|| PlaybackError::new("STVK.PLAYBACK.NO_SOURCE",
            "No supported media URL was returned"))?;
    if !(selected.starts_with("https://") || selected.starts_with("http://")) ||
        selected.contains('\n') || selected.contains('\r') {
        return Err(PlaybackError::new("STVK.PLAYBACK.INVALID_URL",
            "Backend returned an unsafe media URL"));
    }
    Ok(MediaSession {
        content_id: id.to_owned(),
        title: data.get("title").and_then(Value::as_str).unwrap_or(id).to_owned(),
        url: selected.to_owned(),
        is_live: data.get("contentType").and_then(Value::as_str) == Some("live"),
        simulated_live: data.pointer("/demo/simulatedLive").and_then(Value::as_bool)
            .unwrap_or(false),
    })
}

/// Launch the first desktop player adapter.
///
/// This is intentionally a **separate player window**. Integrating AVPlayer
/// surfaces and DRM into the Rust scene graph is a future platform adapter.
pub fn launch(session: &MediaSession) -> Result<String, PlaybackError> {
    let requested = std::env::var("SMARTTVKIT_PLAYER").unwrap_or_else(|_| "auto".into());
    let player = match requested.as_str() {
        "auto" => {
            if available("ffplay") { "ffplay" }
            else if available("mpv") { "mpv" }
            else { "system" }
        }
        "ffplay"|"mpv"|"system" => requested.as_str(),
        _ => return Err(PlaybackError::new("STVK.PLAYBACK.PLAYER_NOT_SUPPORTED",
            "SMARTTVKIT_PLAYER must be auto, ffplay, mpv, or system")),
    };
    let mut cmd = match player {
        "ffplay" => {
            let mut cmd = Command::new("ffplay");
            cmd.args(["-hide_banner", "-loglevel", "warning", "-window_title", &session.title]);
            if session.simulated_live { cmd.args(["-loop", "0"]); }
            else { cmd.arg("-autoexit"); }
            cmd.arg(&session.url);
            cmd
        }
        "mpv" => {
            let mut cmd = Command::new("mpv");
            cmd.arg(format!("--title={}", session.title));
            if session.simulated_live { cmd.arg("--loop-file=inf"); }
            cmd.arg(&session.url);
            cmd
        }
        "system" => {
            #[cfg(target_os="macos")]
            {
                let mut cmd = Command::new("open");
                cmd.args(["-a", "QuickTime Player", &session.url]);
                cmd
            }
            #[cfg(target_os="linux")]
            {
                let mut cmd = Command::new("xdg-open");
                cmd.arg(&session.url);
                cmd
            }
            #[cfg(target_os="windows")]
            {
                let mut cmd = Command::new("cmd");
                cmd.args(["/C", "start", "", &session.url]);
                cmd
            }
            #[cfg(not(any(target_os="macos",target_os="linux",target_os="windows")))]
            {
                return Err(PlaybackError::new("STVK.PLAYBACK.PLAYER_NOT_SUPPORTED",
                    "No system player adapter for this OS"));
            }
        }
        _ => unreachable!()
    };
    cmd.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null())
        .spawn()
        .map_err(|e| PlaybackError::new("STVK.PLAYBACK.PLAYER_LAUNCH_FAILED",
            format!("Could not launch {player}: {e}")))?;
    Ok(format!("Opened {} in {}", session.title, player))
}

fn available(program: &str) -> bool {
    std::env::var_os("PATH").map(|path| {
        std::env::split_paths(&path)
            .any(|part| part.join(program).is_file())
    }).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;
    use std::io::{Read, Write};

    #[test]
    fn project_api_is_config_driven() {
        let conf = r#"{"build":{"applicationId":"unchanged"},"runtime":{"api":{"baseUrl":"http://localhost:8787/v1"},"defaultLocale":"en-AU"}}"#;
        let settings = PlaybackSettings::from_project(conf,Some("ja-JP"),Some("explorer")).unwrap();
        assert_eq!(settings.api_base,"http://localhost:8787/v1");
        assert_eq!(settings.locale,"ja-JP");
        assert!(settings.is_loopback());
        assert_eq!(settings.playback_endpoint("channel-0").unwrap(),
            "http://localhost:8787/v1/playback/channel-0?locale=ja-JP");
    }

    #[test]
    fn refuses_untrusted_content_ids() {
        let settings=PlaybackSettings{api_base:"http://localhost:8787/v1".into(),
            locale:"en-AU".into(),demo_tier:None};
        assert!(settings.playback_endpoint("../secrets").is_err());
        assert!(settings.playback_endpoint("example?token=x").is_err());
    }

    fn serve_once(status: u16, json: &'static str) -> (PlaybackSettings,std::thread::JoinHandle<()>) {
        let socket=TcpListener::bind("127.0.0.1:0").unwrap();
        let addr=socket.local_addr().unwrap();
        let thread=std::thread::spawn(move || {
            let (mut conn,_) = socket.accept().unwrap();
            let mut buffer=[0u8;4096];
            let _=conn.read(&mut buffer).unwrap();
            write!(conn,"HTTP/1.1 {status} Test\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{json}",
                json.len()).unwrap();
        });
        (PlaybackSettings{api_base:format!("http://{addr}/v1"),
            locale:"en-AU".into(),demo_tier:Some("free".into())},thread)
    }

    #[test]
    fn playback_response_selects_progressive_media() {
        let (settings,thread)=serve_once(200,
            r#"{"contentId":"the-moon-is-a-liar","title":"The Moon Is a Liar","contentType":"movie","entitled":true,"source":{"progressive":"http://localhost:8787/_demo/media/ident.mp4","hls":null},"demo":{"simulatedLive":false}}"#);
        let session=resolve(&settings,"the-moon-is-a-liar").unwrap();
        thread.join().unwrap();
        assert_eq!(session.title,"The Moon Is a Liar");
        assert!(session.url.ends_with("ident.mp4"));
        assert!(!session.is_live);
    }

    #[test]
    fn backend_entitlement_error_is_not_hidden() {
        let (settings,thread)=serve_once(403,
            r#"{"error":{"code":"STVK.ENTITLEMENT.UPGRADE_REQUIRED","message":"Requires infinite subscription"}}"#);
        let error=resolve(&settings,"goodbye-third-sun").unwrap_err();
        thread.join().unwrap();
        assert_eq!(error.code,"STVK.ENTITLEMENT.UPGRADE_REQUIRED");
        assert!(error.message.contains("infinite"));
    }

    #[test]
    fn rejects_unsafe_media_url() {
        let (settings,thread)=serve_once(200,
            r#"{"entitled":true,"source":{"progressive":"file:///etc/passwd"}}"#);
        assert_eq!(resolve(&settings,"the-moon-is-a-liar").unwrap_err().code,
            "STVK.PLAYBACK.INVALID_URL");
        thread.join().unwrap();
    }
}
