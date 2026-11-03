# SmartTVKit — High-Level Specification

> **Status:** Proposal for review (v0.1)  
> **Date:** 2026-11-03  
> **Scope:** Open-source framework and reference implementation. Commercial services are described only to establish architectural boundaries.  
> **Implementation status:** No implementation is authorised by this document. Decisions marked *proposed* remain subject to review.

## 1. Vision

**SmartTVKit is an open-source, configuration-driven framework for building, deploying and maintaining television applications across fragmented device ecosystems.**

It aims to let developers build OTT, live-TV, FAST and hybrid broadcast/broadband experiences using a shared application definition, a canonical content and service API, and reusable platform-specific runtimes.

The primary product is **not** a consumer streaming service, media host or transcoder. It is an application development and delivery foundation.

A future optional SmartTVKit SaaS will provide visual/AI-assisted application design, hosted backend adapters, managed builds, device compatibility intelligence and release operations. The open-source project must remain functional without that SaaS.

### One-sentence developer promise

> Describe a TV application in validated JSON, connect it to a canonical data API, and run it on supported devices without rebuilding business logic for each platform.

## 2. Goals and non-goals

### Goals

1. **Broad device reach:** reuse as much behaviour and application definition as practical across web, HbbTV, Samsung, LG, Android TV, Fire TV, tvOS, Roku and future targets.
2. **Configuration-first:** support substantial UI and workflow changes through versioned declarative configuration rather than client binary updates.
3. **Backend agnosticism:** clients consume SmartTVKit canonical contracts; customer backend transformation belongs in adapters/gateways.
4. **Broadcast + OTT convergence:** treat DVB-I service discovery, live TV, programme information and HbbTV as first-class product domains, not optional afterthoughts.
5. **Open-source independence:** developers can run, configure, test, build and self-host the framework without subscribing to managed services.
6. **AI-ready authoring:** publish schemas, examples, component descriptions and machine-readable validation; support optional MCP tooling for local or hosted development.
7. **Production discipline:** design for TV remote navigation, hardware playback, security, low-resource devices, observability and store policy constraints.
8. **Community compatibility feedback:** accept reproducible device/firmware reports and turn findings into regression tests and capability records.

### Non-goals (initially)

- Building a CDN, video transcoder, DRM licence service, content-rights platform, ad server, subscriber billing system or full CMS.
- Guaranteeing zero app-store resubmissions or zero certification effort.
- Guaranteeing one binary, one UI rendering technology or identical device capabilities on every platform.
- Acting as a native DVB-I system receiver, controlling RF tuners or replacing manufacturer-specific channel guides without the requisite APIs/permissions.
- Supporting arbitrary user-provided executable JavaScript or native plug-ins through remote configuration.
- Implementing every platform in the first milestone.

## 3. Users and primary use cases

| Persona | Core task |
| --- | --- |
| Streaming application engineer | Clone project, choose platform, configure UI, connect a backend and build a functional TV app. |
| Broadcaster / hybrid-TV integrator | Import DVB-I service discovery and service-list metadata, show channels/EPG, and launch compatible playback. |
| OTT platform / agency | Create branded applications for multiple content clients with common runtime and repeatable builds. |
| Open-source maintainer | Reproduce a device defect, add a capability rule or regression test, and publish an upstream fix. |
| Future SaaS customer | Use a visual editor or AI tool to update an application, preview and validate, then generate/release platform deliverables. |

Representative experiences: content catalogue and VOD playback; FAST/live channel browsing; seven-day EPG; DVB-I service-list discovery; entitlement-aware playback; remote-configured navigation, branding and layout.

## 4. Architectural principles

1. **Stable shell, evolving application definition.** Device-native integration remains in a relatively stable shell; presentation and supported workflows are declared remotely where permitted.
2. **Data contract at the boundary.** Client rendering components never depend directly on customer-specific CMS response structures.
3. **One authoring contract, multiple runtimes.** A single schema can be interpreted by platform renderers, with explicit capability differences and fallbacks.
4. **Explicit capabilities, not optimistic assumptions.** Features are enabled according to declared capabilities for a concrete runtime, OS and device.
5. **No arbitrary code in configuration.** Configuration references an approved component, expression and action catalogue.
6. **Open-source parity of semantics.** Locally authored and SaaS-authored app definitions have the same meaning.
7. **Platform playback is authoritative.** Use hardware-supported player/DRM stacks rather than building a universal video decoder in Rust.
8. **Versioning and rollback from day one.** Schemas, runtimes, configuration, adapters and published releases have independent identities.
9. **Accessibility, remote focus and performance are core concerns.**

## 5. Conceptual system architecture

```text
            Developer CLI / local MCP / future SaaS Studio
                              |
                     Application Definition
                    (versioned JSON + assets)
                              |
           +------------------+------------------+
           |                                     |
    Configuration / release                Canonical APIs
    distribution and cache                       ^
           |                                     |
           v                              Open adapter gateway
    Platform app runtime                          ^
    +------------------------+         +----------+--------------+
    | config interpretation  |         | Customer CMS / Auth     |
    | UI + state + actions   |         | DVB-I service lists    |
    | focus and navigation   |         | EPG / entitlement      |
    | playback orchestration |         +-------------------------+
    +------------------------+
           |
    Platform shell / renderer / native player / OS APIs
           |
    TV hardware, browser or HbbTV runtime
```

### Major subsystems

- **Application definition:** declarative UI, routes, actions, bindings, themes, data requests, capability constraints and feature flags.
- **Core runtime:** validates/interprets definitions, owns logical app state, navigation intent, action dispatch and playback orchestration.
- **Platform adapter:** rendering, remote input, focus integration, player/DRM, storage, network, lifecycle and logging.
- **Canonical API and gateway:** stable device-facing resources, adapters for disparate backends, cache and policy enforcement.
- **Tooling:** schema generation, validation, preview, developer CLI, test fixtures and optional MCP server.
- **Compatibility registry:** device/OS/firmware/capability observations, tested workarounds and regressions.

The architecture distinguishes a **control plane** (app definitions, build/release metadata) from a **data plane** (catalogue, EPG, sessions, playback resolution). Video segments should generally flow directly from the appropriate CDN to the native player.

## 6. Application definition and UI language

### Approach

- Use **JSON**, not YAML, as the canonical application definition format.
- Define a SmartTVKit **JSON Schema** and publish machine-readable component, action and capability catalogues.
- Borrow from **A2UI**: declarative component IDs, a component catalogue, binding, actions and separation of view structure from data. SmartTVKit is initially **A2UI-inspired**, not automatically wire-compatible with the upstream A2UI protocol.
- Use **DTCG design tokens** for themes where appropriate, **JSON Pointer** for simple bindings, and evaluate a safe restricted expression language such as **CEL** for conditional behaviour.
- Prefer immutable, versioned application definition snapshots at startup. Incremental JSON Patch-style updates may be added later.
- All syntax and built-in functions must be deterministic and portable across target runtimes.

### Configuration concerns

| Domain | Examples |
| --- | --- |
| Application metadata | IDs, schema version, runtime compatibility, locale, required capabilities |
| Routes/screens | Initial route, route table, state restoration, transitions |
| UI components | Hero, content rail, collection grid, search, details, channel guide, player overlays, modal |
| Layout/theming | Design tokens, breakpoints/safe area, typography, styling, media assets |
| Focus | Initial focus, directional navigation rules, focus restoration, virtualised lists |
| Data binding | Refer to canonical objects, selectors, loading/error/empty states |
| Actions | Navigate, select content, play, seek, switch channel, request data, authenticate |
| Conditions | Entitlements, locale, form factor, capability, experiments |
| Playback UI | Controls, default track preferences, error and retry presentation |
| Broadcast UI | Channel lists, EPG representation, service metadata, DVB-I sources |

**Important limitation:** configuration only activates capabilities already shipped and approved in the installed runtime. A new native API, DRM mechanism or unsupported component still requires a new appropriate runtime release, and policy review may be necessary even for some remote feature changes.

### Example (illustrative, NOT an approved schema)

```json
{
  "schemaVersion": "0.1-draft",
  "application": { "id": "sample-tv", "title": "Sample TV" },
  "requiredCapabilities": ["navigation.dpad", "playback.hls"],
  "initialRoute": "home",
  "screens": {
    "home": {
      "root": "homeColumn",
      "components": [
        { "id": "homeColumn", "component": "Column", "children": ["featured"] },
        { "id": "featured", "component": "ContentRail",
          "items": { "path": "/catalogue/featured" },
          "onSelect": { "action": "openDetails" } }
      ]
    }
  }
}
```

The first schema RFC must decide whether to use A2UI's flat adjacency-list model verbatim, how state and async resources are expressed, and how focus metadata is attached. Do not treat this sketch as a contract.

## 7. Canonical data model and adapter gateway

The app runtime sees **versioned SmartTVKit domain contracts**, independent of source vendor:

- **Catalogue:** Content, Series, Season, Episode, Collection, Artwork, Genre, Availability.
- **Live and broadcast:** Service, Channel, Programme, Schedule/EPG, ServiceList, ServiceInstance.
- **Identity:** User, Profile, Session, Preferences, Entitlement (privacy and access scoped).
- **Playback:** PlaybackRequest, PlaybackSource, DRMRequirements, AudioTrack, SubtitleTrack, ResumePosition, PlaybackError.
- **Cross-cutting:** stable IDs, localisation, pagination, timestamps, provenance, typed capability/extension fields.

Requirements:

1. Publish JSON Schema/OpenAPI contracts for the canonical API, independently of individual source adapters.
2. Normalise source differences **in an adapter gateway**, not in platform renderers or screen definitions.
3. Support both **open-source self-hosted adapters** and optional **SaaS-hosted adapters** over the same client-facing contract.
4. Permit reusable connector packages and customer-specific transformation rules; maintain namespaced extensions so DVB-I-specific metadata is not discarded.
5. Never expose backend API credentials, service tokens or transformation secrets to a client configuration.
6. Support per-tenant isolation, bounded execution and outbound-network allowlists if future SaaS allows custom adapter code.
7. Avoid forcing all player traffic through the gateway; return authorised stream/DRM metadata and use appropriate direct playback paths.
8. Define cache and failure behaviour for catalogue, EPG and service lists; minimise repeated login and expensive round trips.

**Boundary:** the core open-source project should include a reference adapter gateway and sample adapters/fixtures. The commercial service may operate that gateway and provide advanced managed integration tools, but must not own an incompatible private data format.

## 8. DVB-I and HbbTV

DVB-I support is a first-class differentiator, with a distinct standards-aware module.

### Initial DVB-I capability goals

- Resolve a configured DVB-I entry point and retrieve/validate service lists.
- Represent services, service instances, metadata and associated programme information without lossy flattening.
- Merge suitable DVB-I services into a familiar channel list and EPG user experience.
- Select playback delivery options according to available device capabilities and metadata.
- Diagnose malformed or unsupported metadata, including reproducible sample fixtures.
- Separate **DVB-I service discovery and application experiences** from **native TV receiver/tuner integration**, which requires appropriate device support.

### HbbTV capability goals

- Provide a browser-based HbbTV-facing application where available APIs permit.
- Handle HbbTV-specific remote-control, application lifecycle, media and broadcast signalling constraints through adapters.
- Support older HbbTV engines with capability detection and graceful fallbacks; do **not** assume WASM availability across installed devices.
- Validate actual standards profiles and target devices, rather than claiming blanket HbbTV conformance.

DVB-I alignment is intended with **ETSI TS 103 770 / DVB BlueBook A177** and implementation guidelines. HbbTV target versions and claimable conformance must be specified per tested implementation.

## 9. Platform support policy

**Support is phased, capability-based and evidence-driven.** A platform is not “supported” until a reproducible build, remote navigation, media playback, and basic lifecycle tests pass on representative hardware (or an appropriately scoped officially supported runtime).

| Platform family | Proposed route | Initial status |
| --- | --- | --- |
| Desktop web/browser | Web renderer + Rust/WASM where appropriate | First proof target |
| HbbTV web runtimes | Web/HbbTV adapter; JS fallback where WASM unavailable | Early DVB-I proof target |
| LG webOS | Web application; platform playback integration | Early validation |
| Samsung Tizen | Packaged web application; platform playback integration | Early validation |
| Android TV / Fire OS | Android shell; Rust native core through FFI/JNI; native player | Early validation |
| Apple tvOS | Native shell/renderer + shared Rust library where useful | Later |
| Roku | Separate renderer/runtime implementation using shared schemas/contracts; no assumed Rust/WASM support | Later |
| Amazon Vega OS | Investigate approved native/React Native integrations and remote update rules | Research |
| VIDAA, Xbox, PlayStation, other embedded TVs | Feasibility, device access and distribution dependent | Research |

The core may be implemented in Rust, but **Rust execution on every target is not a prerequisite**. The invariant is the observable behaviour of the JSON schema, canonical data contract and compatibility tests.

TV UI rendering strategy (web vs platform-native vs cross-platform framework) is an explicit technical spike, not an already-made commitment.

## 10. Playback and device integration

Platform adapters own the native media player, DRM negotiation and OS interfaces. The common core owns playback *intent*, lifecycle coordination and user-facing state.

Minimum playback behaviours to standardise:

- playback request/resolution; HLS as the initial working format, DASH where required by target platform/use case;
- play/pause/seek, resume, track selection, buffering and error states;
- live stream start/recovery, channel switching and EPG-to-playback actions;
- capability checks for codecs, DRM, subtitle formats and stream delivery;
- graceful degradation and clear unsupported-content outcomes.

DRM support, licensing and media interoperability must be verified per device and content provider. Do not imply that having a common playback API makes DRM portable.

## 11. Configuration, runtime updates and app-store constraints

Maintain three independently versioned artifacts:

1. **Store/OS application shell:** signed platform integration and bundled fallback.
2. **Application runtime:** Rust/WASM/JS/native logic, updated remotely **only where platform distribution policies and approved application architecture permit**.
3. **Application definition and customer data mappings:** declarative configuration + server-side adapter changes, updated independently subject to device compatibility and store rules.

Every boot should determine its app ID, shell/runtime API version, declared capabilities and last-known-good configuration, and fetch a compatible release manifest when online.

Expected features:

- immutable, signed or hash-verified manifests/assets and TLS;
- schema/runtime/capability compatibility checks before activation;
- pinning, staged rollouts, canaries and rollback;
- cached last-known-good app definition and an operable fallback when the SaaS/control plane is unavailable;
- no unsupported “self-modifying” store binary scheme;
- predictable reporting when a platform requires new certification.

**Goal:** minimise unnecessary store updates, **not** promise to bypass review policies. Changes introducing functionality may still require review even when represented as JSON; platform policy is part of release validation.

## 12. Developer experience, documentation and MCP

All developers should be able to:

1. create or import an application definition;
2. run `validate` with precise schema, component, binding, focus and capability errors;
3. preview it locally with sample canonical data;
4. run a sample gateway/backend locally;
5. build a test artifact for an implemented target;
6. view compatibility and diagnose a failing device.

Publish:

- JSON Schemas, component/action/capability catalogues and canonical API contract;
- working sample app definitions, tests and adapter fixtures;
- clear versioning/migration notes and architecture decision records;
- an AI-readable getting-started guide with concise examples and relevant reference links.

An **optional local MCP server** should expose schema discovery, component lists, read/patch draft, validation and preview tools. A future hosted MCP server can additionally expose tenant-scoped application, build and release services, using the **same schema**. No single LLM provider is required.

Publication/production changes are separate from drafting and require authenticated authorisation and explicit confirmation. Prompts and external adapter responses are untrusted inputs.

## 13. Security, privacy and operational resilience

- Separate app configuration from secrets. Never ship private signing keys, server API credentials or customer adapter secrets in client artifacts.
- Protect identity/session, token refresh, entitlement and licence acquisition flows per platform.
- Validate and size-limit remote JSON; constrain expressions, components, URLs, actions and redirect destinations.
- Verify updates and maintain a trusted rollback; isolate releases by customer/application.
- Support minimum-privilege tenant access, audit history and signed release provenance in future SaaS.
- Avoid collecting viewer identities in public GitHub issues. Diagnostics are **opt-in**, redacted and suitable for publication only after review.
- Provide cache, timeout, retry/backoff and observable error handling for control-plane and gateway outages.
- Establish a vulnerability disclosure process and test third-party dependencies/licences.

## 14. Community-driven device compatibility

A key open-source benefit is community-supplied device verification.

Define a reusable compatibility report containing: device model, OS and firmware version, runtime version, app/schema version, tested capability, minimal reproduction steps, expected/actual behaviour and an optional redacted trace.

Maintainers should convert confirmed reports into:

- reproducible regression tests where possible;
- capability records (known-good, known-broken, unverified);
- platform-specific workarounds and release notes.

Community reports are **evidence**, not certification. Critical playback/DRM/store behaviours require owned-device or trusted partner validation. Never automatically accept an issue report as a production compatibility rule.

## 15. Testing and quality gates

Proposed test layers:

- **Contract tests:** JSON Schema, canonical API shape, migrations, defaulting and error responses.
- **Interpreter tests:** component graph, binding, actions, route state and focus graph; deterministic replay.
- **Adapter tests:** mapping, malformed input, pagination, timeouts and isolation, including DVB-I metadata samples.
- **Playback tests:** supported manifests, source selection, lifecycle and error outcomes.
- **Platform harness:** remote input, focus, lifecycle, launch latency, memory pressure and device capability probes.
- **Release tests:** compatible/incompatible configs, signed asset verification, offline fallback, rollout and rollback.

Use CI for deterministic checks; hardware-in-the-loop and manual certification checks remain explicit requirements as platforms mature.

## 16. Open-source structure and commercial boundary

**Existing repository licence: Apache License 2.0.** Maintain an open, independently useful SDK/runtime.

Proposed future repository areas (not yet created):

```text
docs/              High-level spec, RFCs, ADRs, platform guidance
schemas/           Application/UI, component and canonical API schemas
crates/            Rust runtime, validation and DVB-I libraries
platforms/         Platform shells and renderers
gateway/           Reference canonical API and adapters
examples/          Sample OTT, live-TV and DVB-I applications
tools/             CLI, preview, local MCP server, compatibility tooling
tests/             Contract tests and shared conformance fixtures
```

The eventual commercial **SmartTVKit Studio/Cloud** may charge for hosted editing, integrations, builds, signing workflows, release management, monitoring, device testing and support. It must **not** be necessary to run the open-source reference app or use the public format.

No client data or app-definition lock-in: provide export/import and self-hosting documentation.

## 17. Proposed milestones (not committed dates)

### M0 — Specification and proofs

- Approve product principles and architecture boundary.
- Write initial RFCs for application JSON, canonical API, capability manifest and focus navigation.
- Prototype one UI screen rendered from config on desktop web; validate whether A2UI concepts meet TV needs.
- Prototype parsing/validation of a sample DVB-I service list.
- Decide whether Rust/WASM is appropriate for the initial renderer, and record the decision.

**Exit:** a minimal spec and known implementation risks, accepted before framework code expands.

### M1 — Open-source vertical slice

- Working desktop/web reference application from JSON definition.
- D-pad style remote input, screen navigation, content rail and detail screen.
- Canonical mock data and simple self-hostable adapter.
- HLS playback through a browser/native media path where supported.
- Schema validator, error reporting, local development instructions and CI contract tests.

**Exit:** a new engineer can clone, customise and run a sample TV app without managed services.

### M2 — Broadcast and initial device validation

- DVB-I list/service/EPG integration and diagnostics.
- HbbTV execution proof and one or two actual smart-TV web environments.
- Capabilities registry, tests and documented limitations.
- Runtime/config version negotiation and fallback.

**Exit:** demonstrated live TV service browsing and playback on named test devices, with documented limitations.

### M3 — Additional platforms and AI tooling

- Android TV/Fire OS reference shell and native player.
- Local MCP development tools and AI-readable schema catalogue.
- Config migration, preview and further compatibility validation.

**Exit:** independently reproducible builds on multiple distinct platform families.

### M4 — Future managed service (separate project scope)

- Hosted app definitions and adapters, visual editing, build/release orchestration, monitoring.
- Publish a separate SaaS product/security/operations spec before implementation.

## 18. Success criteria

Early success is measured by **independent reuse and portability**, not line count or declared platform count:

- an unfamiliar developer can run a demo without private services;
- UI branding, page structure and navigation can be modified **only in config** and rendered consistently on implemented targets;
- changing an upstream source's field names can be resolved solely in the adapter layer;
- a DVB-I service list can be parsed, validated and presented in a working channel experience;
- unsupported components/capabilities fail validation rather than silently corrupting the UI;
- a broken remote config can be rolled back while a known-good app remains usable;
- an externally reported device issue can be turned into a reproducible report/test.

## 19. Decisions proposed for review

| ID | Proposal | Status |
| --- | --- | --- |
| D-01 | Apache 2.0 open-source project with optional hosted services | Existing licence / proposed product principle |
| D-02 | JSON application definition validated by JSON Schema | Proposed |
| D-03 | Borrow from A2UI; do not promise native protocol compatibility yet | Proposed |
| D-04 | Canonical API + open self-hostable adapter gateway | Proposed |
| D-05 | Rust for reusable core where practical, with platform-specific renderer/shell exceptions | Proposed |
| D-06 | First-class DVB-I/HbbTV module and an early proof target | Proposed |
| D-07 | Remote config broadly; remote executable runtime only where allowed | Proposed |
| D-08 | MCP as an optional developer/management interface, not a client runtime dependency | Proposed |
| D-09 | Broad target matrix with gated/verified support claims | Proposed |
| D-10 | SaaS commercial capabilities kept outside the core OSS dependency path | Proposed |

## 20. Open questions for the next RFCs

1. **UI runtime:** Should the first web renderer use conventional JS/TS components with Rust/WASM logic, a Rust-rendered DOM approach, or another UI engine? Assess remote-focus performance on entry-level TVs.
2. **A2UI compatibility:** Do we use A2UI v0.9.1's component/event wire format, an adapted static-document format, or a conversion layer? Document exactly which parts are compatible.
3. **Focus graph:** Explicit directional edges vs spatial navigation default vs hybrid model; how should focus survive async catalogue updates?
4. **Canonical API:** What is the minimal data contract for the first OTT + DVB-I reference application? Choose IDs, localisation, pagination and optional extensions.
5. **Adapter authoring:** Start with code-based adapters only, or ship a declarative mapping DSL early?
6. **Authentication:** Which session/device-pairing flows must be common, and which are hosted entirely by customers?
7. **Content delivery:** How do DRM and region-specific entitlement flows fit the playback resolver contract?
8. **Support matrix:** Which exact devices/OS versions become M2 acceptance targets, and what is the minimum acceptable boot time?
9. **Distribution:** Which platforms permit hosted assets, and which require packaged-only runtimes for concrete app classes and store policies?
10. **Project organisation:** What governance, contributor policy, release cadence and naming conventions do we want before inviting contributions?

## 21. Standards and reference documents

These are informative starting points, not blanket claims of conformance:

- [A2UI project and specification](https://github.com/a2ui-project/a2ui) — declarative UI, component catalogues and data model; v0.9.1 is the current production family as of this draft.
- [JSON Schema](https://json-schema.org/) — schema validation.
- [RFC 6901 — JSON Pointer](https://www.rfc-editor.org/rfc/rfc6901) — binding pointers.
- [RFC 6902 — JSON Patch](https://www.rfc-editor.org/rfc/rfc6902) — optional incremental changes.
- [DTCG Design Tokens](https://www.designtokens.org/) — common theme token concepts.
- [Common Expression Language](https://cel.dev/) — potential safe conditional expressions.
- [DVB specifications](https://dvb.org/specifications/) and [DVB-I specifications](https://dvb-i.tv/specifications/) — including A177 / TS 103 770 and implementation guidelines.
- [HbbTV specifications](https://www.hbbtv.org/resource-library/specifications/) — including HbbTV 2.0.5 candidate specification.
- [Model Context Protocol](https://modelcontextprotocol.io/) — optional tools/resources interface for AI-assisted authoring.

---

**Review note:** This is a high-level architectural and product proposal. It deliberately does not settle wire schemas, Rust crate design, platform rendering libraries or customer-specific business requirements. Those should be follow-on RFCs **after** this document is reviewed.
