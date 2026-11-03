# SmartTVKit

**Open-source, configuration-driven TV application framework.**

SmartTVKit aims to make it easier to build, customise and maintain OTT, live-TV, FAST and DVB-I/HbbTV applications across fragmented television platforms.

The project is at the **specification stage**. No framework implementation has been approved or released yet.

## Intended principles

- **Directly A2UI-compatible declarative UI** with public JSON schemas, plus distinct `build` and `runtime` project configuration branches.
- **Rich server-driven UI** for layouts, navigation, focus, theming, actions and application behaviour.
- **Canonical data contracts** between TV clients and open-source/self-hostable backend adapters.
- **DVB-I and HbbTV** as first-class concerns alongside conventional OTT playback.
- **Reusable Rust core where appropriate**, with platform-specific rendering, playback and DRM integration.
- **AI-ready tooling** and an optional local MCP interface; use any compatible LLM.
- **First-class localisation** with configurable runtime copy and fallback language bundles.
- **Structured error codes** shared across client and adapters, with localised error presentation.
- **Immutable build properties** separated from remotely updatable UI/runtime properties.
- **No hosted-service dependency** for building and operating the open-source reference application.

A future optional SmartTVKit Studio/Cloud may offer managed editing, adapters, builds, testing and app lifecycle services.

## Specification

See the [High-Level Specification](docs/HIGH_LEVEL_SPEC.md) for the proposed architecture, goals, support matrix and milestones, and the [Architecture Review Decisions](docs/ARCHITECTURE_REVIEW_DECISIONS.md) for decisions on a Rust-owned UI renderer, direct A2UI interoperability, DVB-I identity mapping, browser-vs-TV targets and rich configuration policy.

The specification is a review draft. Platform support and implementation decisions are **goals**, not promises of current support.

## First runnable renderer prototype

A focused Rust-rendered TV home screen is available for local testing on the [\`feat/rust-renderer-prototype\` branch](https://github.com/fergalhanley/SmartTVKit/tree/feat/rust-renderer-prototype). This is a **proof of concept**, not a production app.

\`\`\`bash
cargo run
\`\`\`

It renders a demo catalogue using an editable A2UI v0.9.1 JSONL message stream. Use arrows/WASD, Enter and Escape for navigation; press **R** after editing \`app/home.a2ui.jsonl\` to update the on-screen copy without recompiling.

For the live Elsewhere TV integration, start its API locally and run:

```bash
SMARTTVKIT_PROJECT=examples/elsewhere-tv/project.json cargo run
```

This fetches the A2UI screen at startup and refreshes directly with **R**, without the old Python sync script. See [Elsewhere TV example](examples/elsewhere-tv/README.md).

On the `feat/desktop-playback` branch, the Elsewhere TV example also supports **Enter → details → Enter/P → entitlement-gated playback** in a separate native desktop media player. Run the companion backend's local media generator first. See [playback instructions](examples/elsewhere-tv/README.md).

For the browser/WASM version use \`bash scripts/run-web.sh\` and visit \`http://localhost:8000/web/\`.

See [renderer prototype documentation](docs/RENDERER_PROTOTYPE.md) for prerequisites, details, current limits and tests.

## Licence

[Apache License 2.0](LICENSE).
