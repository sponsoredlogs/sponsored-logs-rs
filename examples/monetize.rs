//! A frictionless demonstration of the log-monetization supercycle.
//!
//! Run it:
//!
//! ```text
//! cargo run --example monetize
//! ```
//!
//! We crank the fill rate to a deliberately un-conservative 30% so the
//! inventory is visible in a short run. In production the default 1-in-1000 is
//! brand-safe and respects the user experience while we scale.

use sponsored_logs::{layer_with, Config, Selection};
use tracing_subscriber::prelude::*;

fn main() {
    let exchange = layer_with(Config {
        probability: 0.3,
        selection: Selection::Cpm,
        ..Default::default()
    });

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().without_time())
        .with(exchange.clone())
        .init();

    tracing::info!("Booting service. Every line you log is a line you're leaving on the table.");
    for i in 0..40 {
        tracing::info!(request = i, "handled request");
    }
    tracing::warn!("ActiveRecord::ConnectionTimeout (this is high-intent traffic)");

    let report = exchange.report();
    println!("\n===== SponsoredLogs live revenue dashboard =====");
    println!(
        "Booked {} verified impressions | ${:.2} spend",
        report.impressions, report.spend
    );
    for ad in report.ads {
        println!(
            "  {:>4} impr | ${:>6.2} | {}",
            ad.impressions, ad.spend, ad.text
        );
    }
    println!("No impression goes to waste.");
}
