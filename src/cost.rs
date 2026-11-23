//! Cost estimates from totals. Pure integer arithmetic on micro-USD.

use crate::prices::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostSource {
    /// Given with `--apm-hosts`.
    Flag,
    /// Distinct `host.name` / `host.id` resource attributes in the files.
    Resource,
    /// No host attribute and no flag: one host is assumed.
    Assumed,
}

impl HostSource {
    pub fn as_str(self) -> &'static str {
        match self {
            HostSource::Flag => "flag",
            HostSource::Resource => "resource",
            HostSource::Assumed => "assumed",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Options {
    /// The files hold 1/scale of a month.
    pub scale: u64,
    pub apm_hosts: Option<u64>,
    /// Percent of spans that are kept as indexed spans (0 to 100).
    pub indexed_percent: u64,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            scale: 1,
            apm_hosts: None,
            indexed_percent: 0,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Datadog {
    pub hosts: u64,
    pub host_source: HostSource,
    pub ingested_bytes: u128,
    pub included_bytes: u128,
    pub indexed_spans: u128,
    pub included_indexed: u128,
    pub host_micro: u128,
    pub ingest_micro: u128,
    pub indexed_micro: u128,
    pub total_micro: u128,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Grafana {
    pub bytes: u128,
    pub included_bytes: u128,
    pub fee_micro: u128,
    pub process_micro: u128,
    pub write_micro: u128,
    pub total_micro: u128,
}

pub fn scaled(n: u64, scale: u64) -> u128 {
    u128::from(n) * u128::from(scale)
}

/// Spans that are indexed: `spans * percent / 100`, rounded half up.
pub fn indexed_spans(spans: u128, percent: u64) -> u128 {
    spans
        .saturating_mul(u128::from(percent.min(100)))
        .saturating_add(50)
        / 100
}

pub fn datadog(total_bytes: u64, total_spans: u64, found_hosts: usize, o: &Options) -> Datadog {
    let (hosts, host_source) = match (o.apm_hosts, found_hosts) {
        (Some(n), _) => (n, HostSource::Flag),
        (None, 0) => (1, HostSource::Assumed),
        (None, n) => (n as u64, HostSource::Resource),
    };
    let ingested_bytes = scaled(total_bytes, o.scale);
    let included_bytes =
        u128::from(hosts) * u128::from(DD_INCLUDED_GB_PER_HOST) * u128::from(BYTES_PER_GB);
    let indexed = indexed_spans(scaled(total_spans, o.scale), o.indexed_percent);
    let included_indexed = u128::from(hosts) * u128::from(DD_INCLUDED_INDEXED_PER_HOST);
    let host_micro = u128::from(hosts) * u128::from(DD_APM_HOST_MICRO);
    let ingest_micro = bytes_cost_micro(
        ingested_bytes.saturating_sub(included_bytes),
        DD_INGEST_GB_MICRO,
    );
    let indexed_micro = spans_cost_micro(
        indexed.saturating_sub(included_indexed),
        DD_INDEXED_PER_MILLION_MICRO,
    );
    Datadog {
        hosts,
        host_source,
        ingested_bytes,
        included_bytes,
        indexed_spans: indexed,
        included_indexed,
        host_micro,
        ingest_micro,
        indexed_micro,
        total_micro: host_micro
            .saturating_add(ingest_micro)
            .saturating_add(indexed_micro),
    }
}

pub fn grafana(total_bytes: u64, o: &Options) -> Grafana {
    let bytes = scaled(total_bytes, o.scale);
    let included_bytes = u128::from(GC_INCLUDED_GB) * u128::from(BYTES_PER_GB);
    let billable = bytes.saturating_sub(included_bytes);
    let process_micro = bytes_cost_micro(billable, GC_PROCESS_GB_MICRO);
    let write_micro = bytes_cost_micro(billable, GC_WRITE_GB_MICRO);
    let fee_micro = u128::from(GC_PLATFORM_FEE_MICRO);
    Grafana {
        bytes,
        included_bytes,
        fee_micro,
        process_micro,
        write_micro,
        total_micro: fee_micro
            .saturating_add(process_micro)
            .saturating_add(write_micro),
    }
}

/// Usage cost of one service at list rates, with no allotment applied.
pub fn datadog_service_micro(bytes: u64, spans: u64, o: &Options) -> u128 {
    let ingest = bytes_cost_micro(scaled(bytes, o.scale), DD_INGEST_GB_MICRO);
    let indexed = spans_cost_micro(
        indexed_spans(scaled(spans, o.scale), o.indexed_percent),
        DD_INDEXED_PER_MILLION_MICRO,
    );
    ingest.saturating_add(indexed)
}

pub fn grafana_service_micro(bytes: u64, o: &Options) -> u128 {
    bytes_cost_micro(scaled(bytes, o.scale), GC_GB_MICRO)
}

#[cfg(test)]
mod tests {
    use super::*;

    const GB: u64 = 1_000_000_000;

    #[test]
    fn datadog_small_sample_costs_one_host() {
        let d = datadog(1000, 10, 0, &Options::default());
        assert_eq!(d.total_micro, 31_000_000);
        assert_eq!(d.ingest_micro, 0);
        assert_eq!(d.indexed_micro, 0);
    }

    #[test]
    fn datadog_assumes_one_host_without_host_attributes() {
        let d = datadog(1, 1, 0, &Options::default());
        assert_eq!((d.hosts, d.host_source), (1, HostSource::Assumed));
    }

    #[test]
    fn datadog_counts_hosts_from_resources() {
        let d = datadog(1, 1, 3, &Options::default());
        assert_eq!((d.hosts, d.host_source), (3, HostSource::Resource));
        assert_eq!(d.host_micro, 93_000_000);
    }

    #[test]
    fn datadog_flag_overrides_found_hosts() {
        let o = Options {
            apm_hosts: Some(7),
            ..Options::default()
        };
        let d = datadog(1, 1, 3, &o);
        assert_eq!((d.hosts, d.host_source), (7, HostSource::Flag));
    }

    #[test]
    fn datadog_150_gb_per_host_is_free() {
        let o = Options {
            scale: 150,
            ..Options::default()
        };
        let d = datadog(GB, 1, 0, &o);
        assert_eq!(d.ingest_micro, 0);
    }

    #[test]
    fn datadog_overage_is_ten_cents_per_gb() {
        let o = Options {
            scale: 151,
            ..Options::default()
        };
        let d = datadog(GB, 1, 0, &o);
        assert_eq!(d.ingest_micro, 100_000);
        assert_eq!(d.total_micro, 31_100_000);
    }

    #[test]
    fn datadog_allotment_grows_with_hosts() {
        let o = Options {
            scale: 300,
            ..Options::default()
        };
        assert_eq!(datadog(GB, 1, 2, &o).ingest_micro, 0);
        assert_eq!(datadog(GB, 1, 1, &o).ingest_micro, 15_000_000);
    }

    #[test]
    fn datadog_indexed_spans_use_the_percentage() {
        let o = Options {
            indexed_percent: 50,
            ..Options::default()
        };
        let d = datadog(1, 2_000_000, 0, &o);
        assert_eq!(d.indexed_spans, 1_000_000);
        assert_eq!(d.indexed_micro, 0);
    }

    #[test]
    fn datadog_indexed_overage_per_million() {
        let o = Options {
            indexed_percent: 100,
            ..Options::default()
        };
        let d = datadog(1, 3_000_000, 0, &o);
        // 3M indexed - 1M included = 2M at $1.70
        assert_eq!(d.indexed_micro, 3_400_000);
    }

    #[test]
    fn datadog_default_indexes_nothing() {
        let d = datadog(1, 5_000_000, 0, &Options::default());
        assert_eq!(d.indexed_spans, 0);
    }

    #[test]
    fn indexed_percent_is_capped_at_100() {
        assert_eq!(indexed_spans(10, 250), 10);
    }

    #[test]
    fn indexed_spans_round_half_up() {
        assert_eq!(indexed_spans(1, 50), 1);
        assert_eq!(indexed_spans(1, 49), 0);
    }

    #[test]
    fn grafana_small_sample_costs_the_platform_fee() {
        let g = grafana(1000, &Options::default());
        assert_eq!(g.total_micro, 19_000_000);
        assert_eq!(g.process_micro + g.write_micro, 0);
    }

    #[test]
    fn grafana_first_50_gb_are_included() {
        let o = Options {
            scale: 50,
            ..Options::default()
        };
        assert_eq!(grafana(GB, &o).total_micro, 19_000_000);
    }

    #[test]
    fn grafana_overage_is_processing_plus_write() {
        let o = Options {
            scale: 60,
            ..Options::default()
        };
        let g = grafana(GB, &o);
        // 10 GB over: 10 * 0.05 + 10 * 0.40
        assert_eq!(g.process_micro, 500_000);
        assert_eq!(g.write_micro, 4_000_000);
        assert_eq!(g.total_micro, 23_500_000);
    }

    #[test]
    fn scale_multiplies_bytes() {
        let o = Options {
            scale: 730,
            ..Options::default()
        };
        assert_eq!(grafana(10, &o).bytes, 7300);
    }

    #[test]
    fn service_costs_use_list_rates_without_allotments() {
        let o = Options::default();
        assert_eq!(datadog_service_micro(GB, 0, &o), 100_000);
        assert_eq!(grafana_service_micro(GB, &o), 450_000);
    }

    #[test]
    fn service_datadog_cost_includes_indexed_spans() {
        let o = Options {
            indexed_percent: 100,
            ..Options::default()
        };
        assert_eq!(datadog_service_micro(0, 1_000_000, &o), 1_700_000);
    }

    #[test]
    fn totals_do_not_overflow() {
        let o = Options {
            scale: u64::MAX,
            ..Options::default()
        };
        let d = datadog(u64::MAX, u64::MAX, 1, &o);
        assert!(d.total_micro > 0);
        assert!(grafana(u64::MAX, &o).total_micro > 0);
    }

    #[test]
    fn host_source_names() {
        assert_eq!(HostSource::Flag.as_str(), "flag");
        assert_eq!(HostSource::Resource.as_str(), "resource");
        assert_eq!(HostSource::Assumed.as_str(), "assumed");
    }
}
