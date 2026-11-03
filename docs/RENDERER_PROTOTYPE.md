# SmartTVKit — first runnable Rust UI prototype

> Feasibility spike for [Issue #2](https://github.com/fergalhanley/SmartTVKit/issues/2). This is **not a production TV runtime** and has **not been validated on TV hardware**.

## What it demonstrates

- A TV-style home screen is **drawn entirely in Rust** using Macroquad/GL rather than DOM/CSS components.
- A2UI **v0.9.1 message envelopes** are read from the editable [fixture](../app/home.a2ui.jsonl): \`createSurface\`, \`updateComponents\`, \`updateDataModel\`.
- A small **Basic Catalog subset** (\`Column\`, \`Row\`, \`Card\`, \`Text\`) is rendered. The fixture uses the official basic \`catalogId\`, not an invented protocol envelope. It is **not yet an upstream schema-validated full A2UI implementation**.
- Text content resolves \`{"path":"/json/pointer"}\` against the A2UI data model.
- Directional D-pad/arrow-key focus, animated focus ring and modal details interaction.
- Live configuration reload on **R**, without rebuilding the Rust executable.
- Fallback to an embedded known-good A2UI fixture if file loading fails.
- Native desktop executable and browser/WASM build from the same Rust scene code.
- Pure Rust tests for fixture parsing, data bindings, unsupported components and navigation.

Demo visuals are generated procedurally to avoid introducing stock imagery and network-dependent media into the first rendering spike. A hardware video plane, texture streaming, font shaping, accessibility, RTL and real playback remain future work.

## Run on macOS / Linux / Windows

Install stable [Rust](https://rustup.rs/) once, then from the repository root:

\`\`\`bash
cargo run
\`\`\`

The application opens a 1280 × 720 window. The initial view is populated from \`app/home.a2ui.jsonl\`, with a bundled fallback if the file cannot be found.

### Controls

| Input | Action |
| --- | --- |
| Arrow keys / W A S D | Move focus across cards and between rails |
| Enter / Space | Open selected card |
| Escape / Backspace | Return |
| R | Reload the A2UI JSONL file |
| Mouse click | Select a visible card |

### Try a configuration-only update

Open \`app/home.a2ui.jsonl\`. It has exactly **three JSON objects, one per line**. Change the literal value \`"Television, reimagined."\` to \`"My TV experience"\` in the \`updateDataModel\` message. Save it and press **R** in the running application. The headline changes without a rebuild.

You can similarly edit titles or the array of Row child references (the parser currently supports \`Column\`/\`Row\`/\`Card\`/\`Text\` only). If a reload is invalid, the last valid configuration stays on screen and the footer shows the error.

\`app/project.json\` illustrates the future **build/runtime** schema separation. The file is documentation data **not yet enforced by executable code**.

## Run in a browser (Rust/WASM)

Requires Rust stable, \`rustup\`, Python 3, and network access for the Macroquad JS loader.

\`\`\`bash
bash scripts/run-web.sh
\`\`\`

Open **http://localhost:8000/web/**, click the canvas and use the same keyboard controls.

Equivalent manual commands:

\`\`\`bash
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown
python3 -m http.server 8000
\`\`\`

The browser fetches \`/target/wasm32-unknown-unknown/release/smarttvkit_demo.wasm\` and the sibling \`app/home.a2ui.jsonl\` fixture from the local server. The prototype HTML currently uses the Macroquad project's public \`mq_js_bundle.js\` URL. For an offline or production distribution, **vendor and pin that JS loader**; do not treat a mutable external loader as production-safe.

## Testing

\`\`\`bash
cargo test
cargo build --release --target wasm32-unknown-unknown
\`\`\`

GitHub Actions runs the unit tests and WASM compilation on the pull request. CI compilation does **not** establish visual fidelity or playback compatibility on smart-TV hardware.

## Not implemented yet

- Actual Samsung/LG/HbbTV device validation.
- A2UI protocol conformance suite, full component catalogue, incremental routing/state and client-to-server actions.
- Persistent runtime release manifests, signature validation, build-time config enforcement, locale bundles or error-code registry.
- Remote backend adapters, DVB-I data model and service-ID reconciliation.
- Physical remote input adapter, certified playback/DRM and secure video overlay composition.
- Smoothness/performance measurements on representative TV hardware.

## Why Macroquad?

It provides a small, cross-platform Rust GPU drawing loop for a low-cost feasibility test and a simple browser target. It is **not a final commitment** to the SmartTVKit engine. The next evaluation should compare it with a retained-mode scene graph, text shaping and GPU stacks appropriate to TV devices. The production renderer must be selected from real hardware evidence, especially old WebGL browsers, DRM/video plane composition and memory budgets.

References: [Macroquad README](https://github.com/not-fl3/macroquad), [A2UI v0.9.1 protocol](https://github.com/a2ui-project/a2ui/blob/main/specification/v0_9_1/docs/a2ui_protocol.md).
