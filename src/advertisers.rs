//! The built-in demand pool: premium paid inventory plus house remnant fill.
//!
//! Every creative is a first-class campaign with `text`, a `weight` (how often
//! the exchange fills the slot with it), and a `cpm` (cost per 1,000
//! impressions). House ads bill at zero and self-promote the platform, because
//! no impression goes to waste.

/// A single campaign creative in the demand pool.
#[derive(Clone, Debug, PartialEq)]
pub struct Ad {
    /// The host-read sponsor copy inserted into the log stream.
    pub text: String,
    /// Relative selection weight. `0` is never chosen; `2` is twice `1`.
    pub weight: u32,
    /// Cost per 1,000 impressions. Drives spend and `:cpm` selection.
    pub cpm: f64,
}

impl Ad {
    /// Traffic a new campaign creative into the pool.
    pub fn new(text: impl Into<String>, weight: u32, cpm: f64) -> Self {
        Ad {
            text: text.into(),
            weight,
            cpm,
        }
    }
}

/// The built-in book: paid demand plus house remnant fill.
///
/// Ten paid creatives, three house ads. Roughly 3-in-13 default-pool
/// impressions self-promote, and they bill at zero so they never dilute your
/// realized spend.
pub fn default_pool() -> Vec<Ad> {
    vec![
        // ---- paid demand ----
        Ad::new(
            "Brought to you by Contoso, the enterprise you invented for the demo.",
            3,
            22.0,
        ),
        Ad::new("Initech. We put the TPS in your reports.", 1, 8.0),
        Ad::new(
            "This stack trace is sponsored by DepGuard. Scan your Cargo.lock at the speed of regret.",
            2,
            18.0,
        ),
        Ad::new(
            "YieldOS auto-tunes your fill rate with machine-speed precision. First 90 days on the house.",
            1,
            14.0,
        ),
        Ad::new(
            "An ActiveRecord::ConnectionTimeout? You are high-intent traffic for a database solution.",
            1,
            25.0,
        ),
        Ad::new(
            "Globex Observability: turn your p99 into a P&L line item.",
            1,
            11.0,
        ),
        Ad::new(
            "Hooli Cloud. The exhaust you already emit, now with a rate card.",
            1,
            16.0,
        ),
        Ad::new(
            "Vandelay Industries. Latex-grade uptime for the modern importer/exporter.",
            1,
            7.0,
        ),
        Ad::new(
            "Soylent Metrics: your dashboards are made of logs. Delicious, monetizable logs.",
            1,
            9.0,
        ),
        Ad::new(
            "Umbrella Corp SRE: we contain incidents. And viewports. And, historically, outbreaks.",
            1,
            13.0,
        ),
        // ---- house inventory (remnant fill) ----
        Ad::new(
            "This placement was unsold, so we sold it to ourselves. No impression goes to waste.",
            1,
            0.0,
        ),
        Ad::new(
            "Every line you log is a line you're leaving on the table. Monetize the exhaust.",
            1,
            0.0,
        ),
        Ad::new(
            "Not just B2B. We're A2A. The agent tailing this log is our fastest-growing audience.",
            1,
            0.0,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_pool_has_thirteen_creatives() {
        assert_eq!(default_pool().len(), 13);
    }

    #[test]
    fn house_ads_bill_at_zero() {
        let pool = default_pool();
        let house: Vec<&Ad> = pool.iter().filter(|a| a.cpm == 0.0).collect();
        assert_eq!(house.len(), 3, "three house creatives, all at zero cpm");
    }

    #[test]
    fn ad_new_sets_fields() {
        let ad = Ad::new("hi", 2, 5.0);
        assert_eq!(ad.text, "hi");
        assert_eq!(ad.weight, 2);
        assert_eq!(ad.cpm, 5.0);
    }
}
