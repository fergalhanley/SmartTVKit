# SmartTVKit

**Open-source, configuration-driven TV application framework.**

SmartTVKit aims to make it easier to build, customise and maintain OTT, live-TV, FAST and DVB-I/HbbTV applications across fragmented television platforms.

The project is at the **specification stage**. No framework implementation has been approved or released yet.

## Intended principles

- **One declarative JSON application definition**, inspired by A2UI and validated against public schemas.
- **Rich server-driven UI** for layouts, navigation, focus, theming, actions and application behaviour.
- **Canonical data contracts** between TV clients and open-source/self-hostable backend adapters.
- **DVB-I and HbbTV** as first-class concerns alongside conventional OTT playback.
- **Reusable Rust core where appropriate**, with platform-specific rendering, playback and DRM integration.
- **AI-ready tooling** and an optional local MCP interface; use any compatible LLM.
- **No hosted-service dependency** for building and operating the open-source reference application.

A future optional SmartTVKit Studio/Cloud may offer managed editing, adapters, builds, testing and app lifecycle services.

## Specification

See the [High-Level Specification](docs/HIGH_LEVEL_SPEC.md) for the proposed architecture, goals, support matrix and milestones, and the [Architecture Review Decisions](docs/ARCHITECTURE_REVIEW_DECISIONS.md) for decisions on a Rust-owned UI renderer, direct A2UI interoperability, DVB-I identity mapping, browser-vs-TV targets and rich configuration policy.

The specification is a review draft. Platform support and implementation decisions are **goals**, not promises of current support.

## Licence

[Apache License 2.0](LICENSE).
