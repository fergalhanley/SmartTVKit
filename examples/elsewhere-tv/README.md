# Elsewhere TV — SmartTVKit example integration

**Elsewhere TV** is an original fictional multiverse television service built to validate SmartTVKit against a broad test backend. The service backend, Cloudflare deployment, Stream media mappings and hosting configuration belong in a separate private repository. This public example contains only portable client-facing config and a minimal test integration.

## Run the direct API integration

First start the **separate** Elsewhere TV backend from its own checkout:

```bash
node --experimental-strip-types scripts/dev.mjs
```

It serves A2UI at http://localhost:8787/v1/a2ui/home?locale=en-AU.

In the **SmartTVKit** repository on branch `feat/remote-a2ui-loading`:

```bash
SMARTTVKIT_PROJECT=examples/elsewhere-tv/project.json cargo run
```

The Rust application displays its bundled fallback immediately, **fetches the API directly**, validates the returned v0.9.1 messages, then displays the remote catalogue. There is no Python synchronisation step. Press **R** to fetch it again without recompilation. If the backend is unavailable or returns invalid A2UI, the client retains the previous working UI and logs the error.

Switch language (restart the client after changing the locale):

```bash
SMARTTVKIT_PROJECT=examples/elsewhere-tv/project.json SMARTTVKIT_LOCALE=ja-JP cargo run
```

Against a deployed Worker, either change `runtime.api.baseUrl` in `project.json` or override the full endpoint:

```bash
SMARTTVKIT_PROJECT=examples/elsewhere-tv/project.json SMARTTVKIT_A2UI_URL=https://YOUR-WORKER.workers.dev/v1/a2ui/home cargo run
```

To use a local A2UI fixture instead, you can still supply `SMARTTVKIT_A2UI_FILE`. The original `app/project.json` keeps the open-source bundled demo independent of Elsewhere TV.

### Browser/WASM

The browser runtime reads `app/project.json` over HTTP. To enable direct network loading in that build, set `runtime.api.baseUrl` there and run `bash scripts/run-web.sh`. The backend must allow the browser origin through CORS (the local diagnostic runner does). Production deployments should be HTTPS and configure approved origins.

### Legacy fixture sync

The earlier `sync.py` script remains available for offline snapshots and comparison tests, but is **no longer required** for desktop live-backend loading.


## Architectural boundaries

```text
SmartTVKit (open source)              Elsewhere TV (private)
-------------------------             ---------------------------
Rust renderer           <--- A2UI --- Worker /v1/a2ui/home
Canonical client API    <--- JSON --- Worker /v1/catalog, /v1/live, ...
Native playback adapter <--- HLS  --- Stream playback resolver
Config / schema / tests                D1 viewer state + backend mappings
```

The sample `project.json` illustrates the requested split between build-time and runtime configuration, but the prototype does not yet enforce that schema.

## Current limitations

The client now directly loads the configured A2UI endpoint at startup and on manual refresh. It still renders only one scene from a limited A2UI Basic Catalog subset; it is not yet a streaming-data, search, or media-playback client. The renderer's current Basic Catalog subset supports `Column`, `Row`, `Card`, `Text`; it does not yet implement live HLS playback, search, EPG screens, authentication flows or the larger A2UI TV component catalogue.

The service already exposes those backend concepts to drive forthcoming frontend work. It uses **synthetic DVB-I-style identifiers** for mapping tests, not a standards-conformant DVB-I service list. The initial artwork is original procedural placeholder SVG pending AI-generated art.

**The open-source repo must not depend on a privately hosted service to run:** the existing bundled A2UI fixture remains functional independently.
