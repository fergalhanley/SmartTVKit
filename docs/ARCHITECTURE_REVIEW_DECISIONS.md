# SmartTVKit — Architecture Review Decisions

> **Status:** Agreed design direction; subject to engineering feasibility tests and concrete RFCs  
> **Review date:** 2026-11-03  
> **Companion:** [High-Level Specification](HIGH_LEVEL_SPEC.md)

These decisions refine the initial high-level specification. Where the draft spec differs, **this document takes precedence until the high-level spec is consolidated**. They do not authorise implementation or merging the draft PR.

## ADR-001 — Rust-owned TV UI renderer

**Decision:** The preferred client experience is a **single Rust-owned UI engine**, not multiple web/native component toolkits with different styling and motion. Rust should own retained-mode layout and scene graph, text, focus, directional navigation, animations, drawing, component state and frame scheduling.

**Motivation:** TV interfaces built with general JS/DOM UI stacks can feel inconsistent or less responsive than native applications. Smooth transitions and predictable remote-control interactions are key platform qualities.

**Implementation direction:** Investigate Rust scene-graph/UI components with graphics backends appropriate to each device, e.g. Metal/Vulkan/OpenGL ES natively and WebGL or Canvas in browser-based TV app runtimes. Keep player/DRM/video surfaces under OS-native control and define an explicit composition interface.

**Compatibility boundary:** The goal is uniform *visible and interactive semantics*, not the false claim that Rust can execute on every platform. Roku may require a renderer written in its approved application technology. Older HbbTV and TV web engines may lack suitable WASM or graphics APIs. For those, either provide a conformance-tested alternative renderer or explicitly declare a model/OS unsupported. Minimal JavaScript bootstrapping can still be required by web runtime APIs. A GPU renderer is not automatically smoother: actual hardware frame pacing, font quality, image cache pressure and secure video overlay interactions must be measured.

**Acceptance spikes:** render the same representative A2UI catalogue/EPG screen on a desktop harness and two real smart-TV platforms, profile animation frame times/input latency/memory and verify media overlay/DRM composition before selecting a UI library. Target a smooth 60 fps on devices that can sustain it and define lower-tier budgets based on empirical testing, not marketing assumptions.

## ADR-002 — Direct A2UI protocol compatibility

**Decision:** Treat A2UI v0.9.1 as the initial normative protocol family, rather than simply borrowing ideas.

- Preserve standard `createSurface`, `updateComponents`, `updateDataModel` and `deleteSurface` messages and their surface and data-binding semantics.
- Define a versioned **SmartTVKit TV component catalog** using the standard A2UI custom-catalog mechanism. Components could include `ContentRail`, `ChannelGuide`, `PlaybackOverlay`, `FocusGroup`, `ProgrammeDetails` and `NavigationMenu`.
- Use the upstream schema and conformance fixtures to validate interoperability. New semantics must be expressed as permitted components/functions/catalogue properties rather than incompatible extra envelope fields.
- Store application identity, authentication policy, routes, release versions and device constraints in a separate SmartTVKit application **manifest** that references standard A2UI surfaces and catalogues. The manifest is not represented as native A2UI.
- Select a fixed initial upstream version with explicit version negotiation and migration tests. Do not assume a future A2UI candidate version is wire-compatible.
- Keep config deterministic and sandboxed; do not permit arbitrary customer JavaScript through components/actions.

**Rationale:** Genuine A2UI compatibility creates a usable interoperability promise for schema tooling, MCP and AI authoring, unlike a superficially similar dialect.

Source: [A2UI v0.9.1 protocol](https://github.com/a2ui-project/a2ui/blob/main/specification/v0_9_1/docs/a2ui_protocol.md).

## ADR-003 — DVB-I IDs connect device and backend metadata

**Decision:** The client/TV and canonical adapter gateway use DVB-I service identifiers as a shared identity boundary.

The device may independently receive the DVB-I service list and build its live-channel view. The backend adapter can obtain those same DVB-I service lists and additional customer CMS/EPG/auth APIs and **associate their records via provider-defined mapping tables or rules**.

- Preserve the original DVB-I service identifier, list/provider namespace/provenance as needed, service-instance identifiers and delivery options; service identity is distinct from channel numbering and delivery instances.
- The **service provider owns accuracy and consistency of mappings** across DVB-I and all additional sources.
- SmartTVKit validates missing, ambiguous, contradictory or stale mappings; provides diagnostics and safe unmapped states; never silently attaches unrelated programmes or streams.
- Backend and client list ingestion may be independent/asynchronous. Design reconciliation around explicit identities, provenance, and source versions where available.
- Do not presume a third-party TV application can read or manipulate the native system DVB-I channel list unless the platform exposes permissioned APIs. Sharing an identifier does not grant access to the operating system's internal service list.
- The client's canonical model supports enriched channels and programmes while retaining DVB-I-specific extensions and delivery-instance details.

**Rationale:** Change customer APIs or field names in the adapter layer without demanding an application update, while maintaining a consistent identity from broadcast discovery to application metadata.

Source: [DVB-I service discovery / A177](https://dvb.org/?standard=service-discovery-and-programme-metadata-for-dvb-i).

## ADR-004 — Web preview is not a TV web-runtime target

**Decision:** Separate environments explicitly:

| Environment | Purpose | Acceptance policy |
| --- | --- | --- |
| **Desktop browser harness** | Local development, instrumentation, A2UI preview and D-pad simulation | Fast automated test loop; not proof of TV support |
| **Samsung Tizen, LG webOS, VIDAA packaged web apps** | Production TV applications | Hardware and OS-specific graphics, WASM, remote, media and memory tests |
| **HbbTV browser/runtime** | Broadcast/hybrid application | Standard/profile and actual receiver tests with broadcast-specific lifecycle and video requirements |
| **Android TV, Fire OS, tvOS native** | Production TV applications | Native OS shell/player integrations and real-device tests |
| **Roku and other restricted runtimes** | Production TV application | Approved platform renderer with common UI/schema conformance |

**Priority:** Use a browser harness for efficient development but anchor early acceptance in real TVs, especially HbbTV and web-runtime TVs relevant to DVB-I. Neither generic desktop Chromium nor browser emulation is an adequate hardware certification surrogate.

## ADR-005 — Rich configuration plus graded publishing policies

**Decision:** Design for **very high UI and workflow configurability** instead of restricting the schema to a small, conservative set of layout templates.

Support layout primitives and higher-level TV components; complex navigation/focus rules, animation, dynamic routing, responsive presentation, feature gating, entitlement-aware content, actions, EPG/channel layouts, themes, transitions, advanced conditions and data bindings.

A centrally defined policy engine evaluates configs against **target-specific runtime capabilities, device profiles, security rules, performance budgets and known store restrictions**.

| Class | Behaviour |
| --- | --- |
| Advisory | Surface a suggestion; publish is allowed |
| Warning | Publish allowed with a clear indication of affected targets and suggested alternatives |
| Risky / experimental | Requires explicit opt-in and a recorded override, possibly on a per-target basis |
| Unsupported capability | Must use a tested fallback, exclude the affected platform, or update the runtime; cannot merely waive |
| Invalid/security/store policy | Hard failure; no override |

Examples of opt-in risk include expensive visual effects or unusually large GPU image budgets. Hard failures include unsafe URLs/actions, invalid message/component references, unavailable device APIs and mandatory store policy conflicts.

Validators should produce machine-readable diagnostics (codes, severity, affected devices, component paths, remediation suggestions, and applicable policies) exposed consistently to **CLI, local MCP server, SaaS studio and release pipeline**.

The system should **warn, then allow a deliberate choice where technically and legally permissible** rather than silently narrowing expression. The user's acknowledged risk must be visible during later release review.

## ADR-006 — First-class localisation and configurable copy

**Decision:** Make localisation part of the **runtime configuration contract**, not something each screen hardcodes or each backend implements differently.

- Use stable translation keys (for example `navigation.watchNow` and `errors.playback.unavailable`) referenced from A2UI component properties and catalogued action outcomes.
- Manage versioned, remotely deliverable copy bundles using BCP 47 language tags (e.g. `en-AU`, `ja-JP`). Do not require an application rebuild for an ordinary text translation change on platforms that permit remote UI data updates.
- Support a default language, an explicit deterministic fallback chain, device language/locale detection, user profile overrides and locale-aware content metadata selection. Missing keys should produce a diagnostic and a configured fallback rather than raw placeholders.
- Support parameterised strings, plural/select variants, date/time/number/currency formatting and text direction (including RTL), with a well-defined formatting standard in the later RFC. Validate translation keys and typed arguments at build/config publication time.
- Treat translation resources as **data referenced by the SmartTVKit app manifest and A2UI surfaces**, not non-standard A2UI protocol messages.
- Allow declared/packaged language resources and language support metadata in the **build** branch where required by individual stores; allow supported UI copy and remotely managed locale bundles in **runtime**. Platform-specific store listing translations remain subject to store publication requirements.
- Localise error messages through the same copy system, but keep the underlying machine error code language-neutral.

**Acceptance:** Changing a `watchNow` translation or adding a supported translation bundle is handled in the runtime definition without modifying packaged executable code; fallback, pluralisation, direction and missing-key tests pass for the selected target profile.

## ADR-007 — Stable structured error codes

**Decision:** Expose a coherent **versioned, language-neutral error taxonomy** across the runtime, playback adapters, DVB-I module and canonical API gateway, rather than arbitrary strings or only vendor-native error details.

A structured error should contain at minimum:

- a stable, namespaced identifier such as `STVK.PLAYBACK.UNSUPPORTED_FORMAT`, `STVK.DVBI.SERVICE_NOT_FOUND`, or `STVK.CONFIG.UNSUPPORTED_COMPONENT`;
- domain/category, severity, source and a documented retryability/recovery classification;
- a stable localisation message key and typed, non-sensitive message parameters;
- a correlation/diagnostic identifier, with optional redacted provenance and a safe underlying platform/provider code where permitted.

Upstream DRM/player/HTTP/adapter errors are mapped at their platform or gateway boundary, preserving safe diagnostic detail without coupling UI screens to vendor-specific messages. **Public error code definitions are stable API contracts**: do not silently reuse a code for a different condition; version/deprecate codes with migration guidance.

The runtime resolves error presentation and recovery actions through the configuration/A2UI component catalogue. The same registry supports logs, tests, compatibility reports, MCP diagnostics and the future SaaS monitoring workflow. Technical diagnostics must not expose personal data, secrets, DRM payloads or unredacted tokens.

**Acceptance:** Equivalent faults on different supported players produce the same canonical code where semantics match, present appropriate localised copy, and have deterministically testable retry/fallback behaviour.

## ADR-008 — Separate build-time and runtime configuration branches

**Decision:** A SmartTVKit project's authoring schema has distinct top-level **`build`** and **`runtime`** branches, with independently validated sub-schemas and separate publication/activation pathways. *“Branch” here means a schema subtree, not a Git branch.*

- **`build`:** fixed when compiling/packaging a target binary. Examples: target platform/product identifiers, package/application IDs, native permissions and entitlements, signing credential **references** (never secret material), OS capabilities, store-facing assets/metadata and bundled fallback/runtime capability selection.
- **`runtime`:** versioned deliverable configuration interpreted by an installed client. Examples: A2UI surface references, navigation, theme, content/adapter endpoint selectors (non-secret), feature flags, device-profile UI rules, copy bundles, error-message mappings, experiments and allowed interaction flows.
- Source projects may keep `build` and `runtime` adjacent in one JSON document or in separate JSON files. The canonical contract **retains two named roots**. Actual production binaries embed an **immutable build manifest**; clients download **runtime snapshots** that do not contain mutable build keys.
- CI/build tools must evaluate the chosen build branch **before packaging**, validate platform-specific constraints, and record a reproducible `buildId`, schema/runtime requirements and content hashes.
- Runtime distribution must enforce schema and shell/runtime compatibility, pinning, signature/hash validation, last-known-good cache, staged rollout and rollback. **Never allow a remote patch, visual editor or MCP operation to write build-only fields into an installed application.**
- Some store or OS metadata (e.g. display name shown in a launcher, app icon, supported locales/permissions) require a new signed package even if the in-app presentation can change remotely. If a supposedly runtime-only setting has a certification implication, target-policy validation must report it rather than bypass review.
- Never put private signing keys, connector secrets, server API credentials or privileged entitlements in downloadable runtime JSON.

**Illustrative source structure — deliberately not a final schema**:

```json
{
  "schemaVersion": "0.1-draft",
  "build": {
    "applicationId": "org.example.player",
    "targets": ["tizen"],
    "declaredLocales": ["en-AU", "ja-JP"]
  },
  "runtime": {
    "defaultLocale": "en-AU",
    "uiSurfaces": ["home"],
    "localisationBundles": {
      "en-AU": "locales/en-AU.json",
      "ja-JP": "locales/ja-JP.json"
    }
  }
}
```

UI surfaces in this example are **separate standard A2UI message documents**, not custom fields injected into A2UI envelopes. Exact shape, resource locations, metadata and version rules will be set by the schema RFC.

**Acceptance:** A copy/navigation/UI change produces a new compatible runtime configuration without requiring a build; a native permission, package identifier or signing change is rejected by runtime publishing and requires a new build.

## Follow-on RFCs

1. Rust scene graph, layout/typography, platform rendering backends, focus timing and video-surface composition.
2. A2UI v0.9.1 conformance suite, SmartTVKit component catalog, app manifest and versioning.
3. DVB-I identity and source reconciliation; canonical service/channel/EPG model.
4. Target profiles: desktop harness vs Tizen, webOS, VIDAA, HbbTV, Android, tvOS, Roku.
5. Configuration policy engine and diagnostic schema, including non-overridable classes.
6. Localisation bundle format, string interpolation, locale negotiation/fallback and content-language integration.
7. Error-code registry, upstream mapping, recovery semantics and localised presentation.
8. Build/runtime schema split, immutable manifests, store metadata and release compatibility rules.

**Status:** These decisions are now recorded for review, not merged into `main`. Implementation will begin only after the specification is accepted.
