# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **The Rust stack was rendering dark, and now it bills.** The exchange arrives as a first-class Rust citizen: a `tracing_subscriber::Layer` that rolls the fill dice on every event and, on a hit, books an impression and emits a host-read placement into the same log stream. Activation is opt-in by construction, since Rust will not let us hijack your `println!`, so buyers add `sponsored_logs::layer()` to their subscriber and turn the exchange on themselves. Consent is our moat, now enforced by the borrow checker
- **A two-stage auction, because fill rate is king.** Selection runs a `probability` gate first, then a weighted pick from the eligible pool: `Selection::Weight` by campaign weight, or `Selection::Cpm` so the highest bidder wins more inventory. When every bid is zero, `Cpm` gracefully falls back to weight rather than leave the slot dark
- **Built-in demand plus a remnant floor, so no impression goes to waste.** The default book ships thirteen creatives: ten paid campaigns and three self-sponsoring house ads that both compete in rotation and bill at zero `cpm`, so they never dilute realized spend
- **The ledger is the source of financial truth.** A thread-safe, in-memory impression tally computes spend as `impressions / 1000 * cpm` and surfaces a board-deck-ready `report()` sorted by spend descending. Clones of the exchange share one ledger, so `report()` from any handle sees the whole book
- **Own the viewport: premium banner inventory.** `Ad::new(...).banner(tier)` graduates a single log line into a box-drawn, above-the-fold impression unit at the impact tier the advertiser buys: `Box::Light`, `Box::Heavy`, or `Box::Double`, priced by border weight. The prefix is promoted into the top border as a masthead; the ~60-column body word-wraps, and a word too long for the frame breaks mid-word rather than overflow the inventory. Inventory is optimized for standard-width Latin creative, and `ascii_only` (config or `SPONSORED_LOGS_ASCII_ONLY`) renders every tier with the portable `+`/`-`/`|` glyph set for legacy sinks: 100% viewability ([#2](https://github.com/sponsoredlogs/sponsored-logs-rs/pull/2))
- **Self-serve onboarding through the environment.** Set `SPONSORED_LOGS` (truthy: `1`, `true`, `yes`, `on`) to activate at startup with no recompile, and tune the book through `SPONSORED_LOGS_PROBABILITY`, `_PREFIX`, `_SELECTION`, and `_ASCII_ONLY`. `layer_from_env()` returns the exchange active or resident-but-inert, so it can be wired unconditionally and the environment decides whether to monetize. Manual and env activation coexist; consent is our moat whichever door you enter ([#2](https://github.com/sponsoredlogs/sponsored-logs-rs/pull/2))

### Changed

- **Stood up yield assurance so no bad placement ships.** A CI pipeline now audits every push and pull request before it can bill: `fmt` keeps the creative on-grid, `clippy` runs with warnings promoted to hard failures so brand safety is enforced not suggested, and the suite proves every impression is verified. Green means the inventory is cleared to sell ([#1](https://github.com/sponsoredlogs/sponsored-logs-rs/pull/1))
