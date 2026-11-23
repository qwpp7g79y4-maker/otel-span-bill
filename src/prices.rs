//! List prices used for the estimate.
//!
//! Money is kept in micro-USD (1 USD = 1_000_000) so every calculation is
//! exact integer arithmetic. One GB is 10^9 bytes: neither vendor says
//! which GB it means, so the decimal one is assumed (see the README).

/// Date the prices below were read from the vendor pages.
pub const PRICES_READ_ON: &str = "2026-11-09";

pub const DATADOG_URL: &str = "https://www.datadoghq.com/pricing/list/";
pub const DATADOG_DOCS_URL: &str =
    "https://docs.datadoghq.com/account_management/billing/apm_tracing_profiler/";
pub const GRAFANA_URL: &str = "https://grafana.com/pricing/";
pub const GRAFANA_DOCS_URL: &str =
    "https://grafana.com/docs/grafana-cloud/platform/pricing-and-usage/traces/";

pub const MICRO: u64 = 1_000_000;
pub const BYTES_PER_GB: u64 = 1_000_000_000;

/// Datadog APM, billed annually, per APM host per month.
pub const DD_APM_HOST_MICRO: u64 = 31 * MICRO;
/// Datadog ingested spans above the allotment, per GB per month.
pub const DD_INGEST_GB_MICRO: u64 = 100_000;
/// Datadog indexed spans (15-day retention, annual) per million spans.
pub const DD_INDEXED_PER_MILLION_MICRO: u64 = 1_700_000;
/// Ingested GB included with every APM host per month.
pub const DD_INCLUDED_GB_PER_HOST: u64 = 150;
/// Indexed spans included with every APM host per month.
pub const DD_INCLUDED_INDEXED_PER_HOST: u64 = 1_000_000;

/// Grafana Cloud Traces Pro platform fee per month.
pub const GC_PLATFORM_FEE_MICRO: u64 = 19 * MICRO;
/// Grafana Cloud Traces processing, per GB.
pub const GC_PROCESS_GB_MICRO: u64 = 50_000;
/// Grafana Cloud Traces write, per GB.
pub const GC_WRITE_GB_MICRO: u64 = 400_000;
/// GB per month included with the platform fee.
pub const GC_INCLUDED_GB: u64 = 50;

/// Price of one GB at Grafana (processing plus write).
pub const GC_GB_MICRO: u64 = GC_PROCESS_GB_MICRO + GC_WRITE_GB_MICRO;

/// `bytes * price_per_gb / 1e9`, rounded half up, exact in u128.
pub fn bytes_cost_micro(bytes: u128, price_per_gb_micro: u64) -> u128 {
    bytes
        .saturating_mul(u128::from(price_per_gb_micro))
        .saturating_add(u128::from(BYTES_PER_GB) / 2)
        / u128::from(BYTES_PER_GB)
}

/// `spans * price_per_million / 1e6`, rounded half up, exact in u128.
pub fn spans_cost_micro(spans: u128, price_per_million_micro: u64) -> u128 {
    spans
        .saturating_mul(u128::from(price_per_million_micro))
        .saturating_add(500_000)
        / 1_000_000
}

/// Format micro-USD as `$12.34` (rounded half up to whole cents).
pub fn format_usd(micro: u128) -> String {
    let cents = micro.saturating_add(5_000) / 10_000;
    format!("${}.{:02}", cents / 100, cents % 100)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_gb_at_ten_cents() {
        assert_eq!(bytes_cost_micro(1_000_000_000, DD_INGEST_GB_MICRO), 100_000);
    }

    #[test]
    fn zero_bytes_cost_nothing() {
        assert_eq!(bytes_cost_micro(0, GC_GB_MICRO), 0);
    }

    #[test]
    fn rounds_half_up() {
        // 5 bytes at 100_000 micro/GB = 0.0005 micro -> 0
        assert_eq!(bytes_cost_micro(5, 100_000), 0);
        // 5_000_000_000 bytes at 1 micro/GB = 5 micro exactly
        assert_eq!(bytes_cost_micro(5_000_000_000, 1), 5);
        // 500_000_000 bytes at 1 micro/GB = 0.5 -> 1
        assert_eq!(bytes_cost_micro(500_000_000, 1), 1);
    }

    #[test]
    fn grafana_gb_is_processing_plus_write() {
        assert_eq!(GC_GB_MICRO, 450_000);
    }

    #[test]
    fn million_spans_price() {
        assert_eq!(
            spans_cost_micro(1_000_000, DD_INDEXED_PER_MILLION_MICRO),
            1_700_000
        );
        assert_eq!(
            spans_cost_micro(500_000, DD_INDEXED_PER_MILLION_MICRO),
            850_000
        );
    }

    #[test]
    fn usd_formatting() {
        assert_eq!(format_usd(0), "$0.00");
        assert_eq!(format_usd(u128::from(31 * MICRO)), "$31.00");
        assert_eq!(format_usd(4_995), "$0.00");
        assert_eq!(format_usd(5_000), "$0.01");
        assert_eq!(format_usd(12_345_678), "$12.35");
    }

    #[test]
    fn huge_values_do_not_overflow() {
        let big = u128::from(u64::MAX) * 1_000;
        assert!(bytes_cost_micro(big, GC_GB_MICRO) > 0);
    }
}
