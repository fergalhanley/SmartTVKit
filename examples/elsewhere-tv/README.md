# Elsewhere TV — SmartTVKit example integration

**Elsewhere TV** is an original fictional multiverse television service built to validate SmartTVKit against a broad test backend. The service backend, Cloudflare deployment, Stream media mappings and hosting configuration belong in a separate private repository. This public example contains only portable client-facing config and a minimal test integration.

## Run the first integration

1. Start Elsewhere TV backend locally on port 8787 (using its Node diagnostic runner or Cloudflare Wrangler).
2. In SmartTVKit, on branch `feat/elsewhere-tv-example`, run:

```bash
python3 examples/elsewhere-tv/sync.py
SMARTTVKIT_A2UI_FILE=app/elsewhere-tv.a2ui.jsonl cargo run
```

3. The Rust UI will display catalogue and channel data retrieved from the backend as an A2UI v0.9.1 message stream.
4. After backend content or locale changes, re-run the sync script and press **R** inside the running client. No Rust recompile is needed.

Japanese test:

```bash
ELSEWHERE_TV_LOCALE=ja-JP python3 examples/elsewhere-tv/sync.py
SMARTTVKIT_A2UI_FILE=app/elsewhere-tv.a2ui.jsonl cargo run
```

Deployed API:

```bash
ELSEWHERE_TV_API=https://YOUR-WORKER.workers.dev python3 examples/elsewhere-tv/sync.py
```

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

This is one-off fixture synchronisation, not a true network-driven Rust client. The renderer's current Basic Catalog subset supports `Column`, `Row`, `Card`, `Text`; it does not yet implement live HLS playback, search, EPG screens, authentication flows or the larger A2UI TV component catalogue.

The service already exposes those backend concepts to drive forthcoming frontend work. It uses **synthetic DVB-I-style identifiers** for mapping tests, not a standards-conformant DVB-I service list. The initial artwork is original procedural placeholder SVG pending AI-generated art.

**The open-source repo must not depend on a privately hosted service to run:** the existing bundled A2UI fixture remains functional independently.
