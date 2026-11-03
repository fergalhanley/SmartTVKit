//! A2UI source selection and native HTTP fetching for the renderer prototype.
//! Project JSON is a provisional manifest: it is not the final SmartTVKit schema.
use crate::Scene;
use serde_json::Value;

pub const MAX_A2UI_BYTES: u64 = 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SceneSource {
    Local(String),
    Remote(String),
}
impl SceneSource {
    pub fn label(&self) -> &'static str {
        match self { Self::Local(_) => "LOCAL FILE", Self::Remote(_) => "LIVE API" }
    }
    pub fn path(&self) -> &str {
        match self { Self::Local(p) | Self::Remote(p) => p }
    }
}

/// Resolve the source using the runtime branch only, respecting a locale or URL override.
pub fn from_project(
    text: &str,
    fallback_file: &str,
    locale_override: Option<&str>,
    url_override: Option<&str>,
) -> Result<SceneSource, String> {
    let project: Value = serde_json::from_str(text).map_err(|e| format!("project JSON: {e}"))?;
    let runtime = project.get("runtime").ok_or("project is missing runtime")?;
    let locale = locale_override
        .or_else(|| runtime.get("defaultLocale").and_then(Value::as_str))
        .unwrap_or("en-AU");
    if locale.is_empty() || locale.len() > 35 ||
        !locale.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err(format!("invalid locale: {locale}"));
    }
    let selected_url = if let Some(url) = url_override {
        Some(url.to_string())
    } else if let Some(base) = runtime.get("api")
        .and_then(|v| v.get("baseUrl")).and_then(Value::as_str) {
        let endpoint = runtime.get("a2ui")
            .and_then(|v| v.get("endpoint")).and_then(Value::as_str)
            .unwrap_or("/a2ui/home");
        if endpoint.starts_with("http") || endpoint.contains("..") ||
            endpoint.contains('?') || endpoint.contains('#') {
            return Err("runtime.a2ui.endpoint must be a relative path".into());
        }
        Some(format!("{}/{}", base.trim_end_matches('/'), endpoint.trim_start_matches('/')))
    } else { None };

    if let Some(url) = selected_url {
        if !(url.starts_with("https://") || url.starts_with("http://")) ||
            url.contains('#') || url.contains('\n') || url.contains('\r') {
            return Err("A2UI endpoint must be an HTTP(S) URL without fragments".into());
        }
        let delimiter = if url.contains('?') { '&' } else { '?' };
        return Ok(SceneSource::Remote(format!("{url}{delimiter}locale={locale}")));
    }

    let file = runtime.get("a2ui")
        .and_then(|v| v.get("source")).and_then(Value::as_str)
        .or_else(|| runtime.get("a2uiStream").and_then(Value::as_str))
        .unwrap_or(fallback_file);
    Ok(SceneSource::Local(file.to_owned()))
}

/// Called on a background thread, never on the UI render loop.
#[cfg(not(target_arch = "wasm32"))]
pub fn load_native(source: &SceneSource) -> Result<Scene, String> {
    use std::io::Read;
    let text = match source {
        SceneSource::Local(path) => std::fs::read_to_string(path)
            .map_err(|e| format!("read {path}: {e}"))?,
        SceneSource::Remote(url) => {
            use std::time::Duration;
            let agent = ureq::Agent::config_builder()
                .timeout_global(Some(Duration::from_secs(8)))
                .build()
                .new_agent();
            let mut response = agent.get(url)
                .header("Accept", "application/x-ndjson, application/json")
                .call()
                .map_err(|e| format!("A2UI request failed: {e}"))?;
            let mut text = String::new();
            response.body_mut().as_reader().take(MAX_A2UI_BYTES + 1)
                .read_to_string(&mut text)
                .map_err(|e| format!("A2UI response: {e}"))?;
            text
        }
    };
    if text.len() as u64 > MAX_A2UI_BYTES {
        return Err("A2UI response exceeds 1 MiB".into());
    }
    Scene::from_a2ui_jsonl(&text).map_err(|e| format!("invalid A2UI response: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn direct_source_uses_configured_backend() {
        let project = r#"{"build":{},"runtime":{"defaultLocale":"en-AU","api":{"baseUrl":"http://localhost:8787/v1"},"a2ui":{"endpoint":"/a2ui/home"}}}"#;
        assert_eq!(
            from_project(project, "unused", None, None).unwrap(),
            SceneSource::Remote("http://localhost:8787/v1/a2ui/home?locale=en-AU".into())
        );
        assert_eq!(
            from_project(project, "unused", Some("ja-JP"), None).unwrap(),
            SceneSource::Remote("http://localhost:8787/v1/a2ui/home?locale=ja-JP".into())
        );
    }
    #[test]
    fn bundled_demo_can_stay_local() {
        assert_eq!(
            from_project(r#"{"runtime":{"a2uiStream":"app/home.a2ui.jsonl"}}"#, "x", None, None).unwrap(),
            SceneSource::Local("app/home.a2ui.jsonl".into())
        );
    }
    #[test]
    fn safe_locale_and_overrides() {
        let src = from_project(r#"{"runtime":{}}"#, "x", Some("fr-FR"),
            Some("https://example.test/home?channel=live")).unwrap();
        assert_eq!(src, SceneSource::Remote("https://example.test/home?channel=live&locale=fr-FR".into()));
        assert!(from_project(r#"{"runtime":{}}"#, "x", Some("en-AU&secret=bad"),
            Some("https://example.test/home")).is_err());
        assert!(from_project(r#"{"runtime":{}}"#, "x", None,
            Some("file:///etc/passwd")).is_err());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn fetches_a2ui_over_http() {
        use std::net::TcpListener;
        use std::io::{Read, Write};
        let socket = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = socket.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let (mut conn, _) = socket.accept().unwrap();
            let mut buf = [0; 2048];
            let _ = conn.read(&mut buf).unwrap();
            let data = crate::BUNDLED_A2UI;
            write!(conn, "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                data.len(), data).unwrap();
        });
        let scene = load_native(&SceneSource::Remote(format!("http://{address}/a2ui"))).unwrap();
        server.join().unwrap();
        assert_eq!(scene.headline, "Television, reimagined.");
        assert_eq!(scene.rails.len(), 2);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn rejects_invalid_remote_config() {
        use std::net::TcpListener;
        use std::io::{Read, Write};
        let socket = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = socket.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let (mut conn, _) = socket.accept().unwrap();
            let mut buf = [0; 1024];
            let _ = conn.read(&mut buf).unwrap();
            write!(conn, "HTTP/1.1 200 OK\r\nContent-Length: 4\r\nConnection: close\r\n\r\nbad!").unwrap();
        });
        assert!(load_native(&SceneSource::Remote(format!("http://{address}/bad"))).is_err());
        server.join().unwrap();
    }
}
