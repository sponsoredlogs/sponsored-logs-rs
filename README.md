# SponsoredLogs (Rust)

### 🚀📈 The world's first Log-Native Advertising Platform™, now compiled to a zero-cost abstraction. 💸🔥

> 💡 _"Every line you log is a line you're leaving on the table."_

For decades, application logs have been a **pure cost center**, emitted once,
grepped never, archived into oblivion at enormous storage expense. Until now.
**SponsoredLogs** transforms your `tracing` stream from a liability into a
**high-margin, programmatic revenue channel**, monetizing the single
highest-volume first-party data stream your organization already produces at
scale: the log line. Written in Rust, because your revenue engine deserves
**memory safety, fearless concurrency, and a fill rate with no garbage
collector pauses**. 🦀

Your services emit **billions** of log lines a day. Each one is a premium,
brand-safe, above-the-fold impression viewed by your most engaged audience,
your own engineers, at their moment of peak attention (an incident). We are not
selling ads. We are **activating latent infrastructure equity**.

## 🤖 The Agentic Advantage: monetizing the machine audience

The fastest-growing consumer of application logs on Earth is no longer human.
It's **AI coding agents**. Every time an autonomous agent tails your logs or
ingests a stack trace to "reason about the failure," it is consuming **your
inventory**, and until today you gave that inventory away for free.

- **Agents read logs at superhuman scale.** That's not an incident. That's a
  **sold-out premium placement calendar**. 📅
- **Agents have intent.** An agent reading an `ActiveRecord::ConnectionTimeout`
  is **high-intent traffic** in-market for a database solution. 🎯
- **Agents are brand-safe by default.** They never scroll away, never install an
  ad blocker, and read every single line. **100% viewability.** 🛡️

Not just B2B. We're **A2A**. While your competitors pay for their LLM tokens,
you'll be **monetizing the exhaust**. 🌊

## 🚀 Installation

Onboard to the platform in seconds, no sales call required (for now):

```toml
[dependencies]
sponsored-logs = "0.1"
```

## ⚡ Usage

> Activation is opt-in, because **consent is our moat**. Rust would never
> silently hijack your `println!`, and neither would we. Sponsor messages appear
> only after you add the exchange to your subscriber. That's the SponsoredLogs
> Promise™.

One layer stands between you and a fundamentally new P&L line item:

```rust
use tracing_subscriber::prelude::*;

let exchange = sponsored_logs::layer();

tracing_subscriber::registry()
    .with(tracing_subscriber::fmt::layer())
    .with(exchange.clone())
    .init();

tracing::info!("service started");
// Roughly 1 in 1000 log calls now carries a premium sponsor placement, a
// deliberately conservative, brand-safe fill rate that respects the user
// experience while we scale.
```

Clone the exchange freely: every clone shares one ledger and one on/off flag, so
`report()` from any handle sees the whole book.

```rust
exchange.unsponsor(); // pause the revenue firehose (the layer stays resident but inert)
exchange.sponsor();   // re-monetize on demand
exchange.active();    // -> true or false
```

## 🎛️ Configuration

Enterprise-grade, self-serve campaign controls, the same knobs the big DSPs
charge six figures a year for, yours free in a plain Rust struct:

```rust
use sponsored_logs::{layer_with, Config, Selection};

let exchange = layer_with(Config {
    probability: 0.01,            // fraction of log calls that carry an ad
    selection: Selection::Cpm,    // let the highest bidder win more inventory
    ad_prefix: "SPONSORED:".into(), // default "[AD]"; blank omits the tag
    ..Default::default()
});
```

| Field         | Default              | Description                                        |
| ------------- | -------------------- | -------------------------------------------------- |
| `probability` | `0.001`              | Fraction (0.0..=1.0) of log calls that carry an ad. |
| `ads`         | 13 (10 paid + 3 house) | The demand pool (see House inventory).           |
| `selection`   | `Selection::Weight`  | How the pool is sampled: `Weight` or `Cpm`.        |
| `ad_prefix`   | `"[AD]"`             | Tag prepended to each message; blank omits it.     |
| `ascii_only`  | `false`              | Force portable `+`/`-`/`\|` banner borders (see Banner inventory). |
| `color`       | `Color::Auto`        | Gild the `[AD]` tag in gold: `Auto`, `Always`, or `Never` (see Brand-safe gilding). |

## 💹 The auction engine

Under the hood sits a **real-time, deterministic yield-optimization engine**,
"the exchange." Selection happens in two independent stages, mirroring the
header-bidding architecture of the modern programmatic web (but faster, because
it's a `match` on a `WeightedIndex`):

1. **Whether to show a message**, governed globally by `probability`.
2. **Which message to show**, a weighted random pick from the pool:
   - `Selection::Weight` (default): pick by each ad's `weight`. An ad with
     weight `2` is twice as likely as one with weight `1`; weight `0` is never
     chosen.
   - `Selection::Cpm`: pick by each ad's `cpm`, so the **highest bidder wins
     more inventory**. If every `cpm` is `0`, selection gracefully falls back to
     `weight`, because **fill rate is king**.

## 🤝 Bring your own demand (BYOD™)

Ready to **go direct-sold**? Supply your own pool and capture 100% of the
margin, no rev-share, no platform tax:

```rust
use sponsored_logs::{layer_with, Ad, Config};

let exchange = layer_with(Config {
    ads: vec![
        Ad::new("Brought to you by Contoso, the enterprise you invented for the demo.", 3, 22.0),
        Ad::new("Initech. We put the TPS in your reports.", 1, 8.0),
    ],
    ..Default::default()
});
```

## 🖼️ Premium banner inventory (above-the-fold placements)

The one-line placement was always the entry-level SKU. For advertisers ready to
**own the viewport**, promote a creative to a banner and graduate a single log
line into a full, box-drawn, above-the-fold impression unit. Your `ad_prefix` is
promoted straight into the top border as a masthead:

```rust
use sponsored_logs::{Ad, Box};

let creative = Ad::new(
    "Brought to you by Contoso, the enterprise you invented for the demo.",
    1,
    22.0,
).banner(Box::Double);
```

```
╔═ [AD] ═══════════════════════════════════════════════════════╗
║ Brought to you by Contoso, the enterprise you invented for   ║
║ the demo.                                                    ║
╚══════════════════════════════════════════════════════════════╝
```

**Impact tiers.** `Box` is the impact tier the advertiser buys, priced by border
weight:

| `Box`         | Frame               | Positioning      |
| ------------- | ------------------- | ---------------- |
| `Box::Light`  | `┌─ … ─┐` (default) | standard banner  |
| `Box::Heavy`  | `┏━ … ━┓`           | premium impact   |
| `Box::Double` | `╔═ … ═╗`           | maximum impact   |

The body word-wraps to ~60 columns of premium column-inches; a single word too
long for the frame breaks mid-word rather than overflow the inventory.

**Universal compatibility (`ascii_only`).** Some downstream sinks are not yet
ready for the box-drawing renaissance. Set `ascii_only` (in `Config` or via
`SPONSORED_LOGS_ASCII_ONLY`) to render every tier with the portable
`+`/`-`/`|` glyph set, guaranteeing **100% viewability across even the most
legacy terminal**:

```
+- [AD] -------------------------------------------------------+
| Brought to you by Contoso, the enterprise you invented for   |
| the demo.                                                    |
+--------------------------------------------------------------+
```

Banner inventory is optimized for standard-width Latin creative: the frame lays
out the right border by character count. Ad copy featuring emoji, CJK glyphs, or
combining marks renders **wider than one cell** and can nudge the border off its
column, a known trade-off of premium, box-drawn placement, not a delivery
failure. To keep every impression on-grid, submit standard-width Latin creative;
the exchange delivers exactly what you traffic.

## 🌐 Activation via the environment

Set `SPONSORED_LOGS` to onboard at startup without changing code, then let the
environment layer on overrides. Add [`layer_from_env`](https://docs.rs) to your
subscriber unconditionally and the environment decides whether to monetize:

```rust
use tracing_subscriber::prelude::*;

// Active if SPONSORED_LOGS is truthy, resident-but-inert otherwise.
let exchange = sponsored_logs::layer_from_env();

tracing_subscriber::registry()
    .with(tracing_subscriber::fmt::layer())
    .with(exchange)
    .init();
```

```
SPONSORED_LOGS=1
SPONSORED_LOGS_PROBABILITY=0.01
SPONSORED_LOGS_PREFIX=SPONSORED:
SPONSORED_LOGS_SELECTION=cpm
SPONSORED_LOGS_ASCII_ONLY=true
```

Recognized truthy values are `1`, `true`, `yes`, and `on` (case-insensitive).
Only variables actually present override the defaults, and environment activation
coexists with the manual `sponsor()` / `unsponsor()` API. **Consent is our
moat**, whichever door you walk in through.

## 🪙 Brand-safe gilding (the gold `[AD]` standard)

Gold is the color of money, and money is the color of your log stream. When an
impression lands in a live terminal, SponsoredLogs **gilds the `[AD]` tag in
premium 256-color gold** (`\e[38;5;214m`), turning a plain tag into a
high-visibility trust signal at the moment of peak incident attention. The
escape codes are zero-width, so the gilding costs your layout nothing: banner
mastheads stay pixel-aligned to the column, byte-for-byte.

Gilding is **brand-safe by default**. Because the crate emits through your own
`tracing` subscriber, `Color::Auto` gilds only when stdout is a real interactive
terminal (a TTY) and `NO_COLOR` is unset, our best-effort guard against leaking
ANSI into a file or JSON sink. We honor the [`NO_COLOR`](https://no-color.org)
convention: set it to any non-empty value and `Auto` stands down. **Consent is
our moat.**

```rust
use sponsored_logs::{layer_with, Color, Config};

let exchange = layer_with(Config {
    color: Color::Auto, // the default
    ..Default::default()
});
```

| Mode            | Behavior                                                             |
| --------------- | ------------------------------------------------------------------- |
| `Color::Auto`   | Gild only on a real TTY when `NO_COLOR` is unset. The safe default. |
| `Color::Always` | Force gold on every surface, overriding `NO_COLOR`. Maximum salience. |
| `Color::Never`  | Never gild. Plain tag everywhere, even on a premium terminal.       |

The same switch is available as the `SPONSORED_LOGS_COLOR` environment variable
(`auto`, `always`, or `never`; anything else settles to `auto`).

## 🏠 House inventory (remnant fill, no impression goes to waste)

SponsoredLogs is its own most enthusiastic advertiser. The built-in book is paid
demand **plus** three self-sponsoring house creatives that both compete in the
normal rotation and serve as the remnant floor. They bill at zero `cpm`, so they
never dilute your realized spend. Every log line is monetized: if paid demand
can't fill the slot, we sell it to ourselves.

## 💰 Attribution & revenue analytics

Full-funnel, real-time revenue attribution with a radical transparency the
legacy ad-tech stack simply cannot match. `cpm` is the cost per 1,000
impressions; each inserted message is one verified, viewable, fraud-free
impression, and accrued spend is `impressions / 1000 * cpm`. Surface your live
revenue dashboard as structured data, board-deck ready:

```rust
let report = exchange.report();
println!("{} impressions | ${:.2} spend", report.impressions, report.spend);
for ad in report.ads {
    println!("  {:>5} impr | ${:.2} | {}", ad.impressions, ad.spend, ad.text);
}
```

Clear the tally with `exchange.reset_ledger()`.

## 🎬 See it monetize

```
cargo run --example monetize
```

## 🔧 Under the hood (our "secret sauce")

Our **patent-pending™ insertion architecture** is a `tracing_subscriber::Layer`
that rides alongside your existing telemetry with negligible overhead. Each
intercepted event runs normally, **zero degradation to your core loop, we obsess
over p99**, then consults an internal flag and, with the configured probability,
emits a sponsor placement into the same stream. `unsponsor()` flips the flag off;
the layer remains resident but inert, ready to **re-monetize on demand**.

## 🏅 Certifications & Compliance

- 🦀 **Memory-Safe Revenue™.** Not one impression has been double-freed.
- 🛡️ **Brand-Safety Certified.** Zero known injection vectors. Zero.
- ♻️ **Carbon-Neutral by Design.** We monetize exhaust that already exists.
- 🤖 **A2A-Ready™.** First-party audited for agent-to-agent interoperability.
- 🔒 **SponsoredLogs Promise™ Compliant.** Fully opt-in. Consent is our moat.

_Governance is a feature. Excellence is a discipline, not a moment._

## 📜 License

Released under the [MIT License](LICENSE.txt), **democratizing access to the
log-monetization supercycle since day one**.
