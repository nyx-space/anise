//! Performance regression demonstration: `summary_from_id_at_epoch` does an
//! O(N) linear scan of every segment for the requested NAIF ID on every call,
//! and emits a `debug!` log per non-matching segment. There is no cache of
//! the previously-found segment, so two queries at adjacent epochs in the
//! same segment do redundant work and emit identical spam.
//!
//! Run with `RUST_LOG=debug cargo test --test perf_segment_cache -- --nocapture`
//! to also see the per-non-match debug lines.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Once;

use anise::constants::frames::EARTH_ITRF93;
use anise::prelude::*;
use hifitime::Epoch;
use log::{Level, LevelFilter, Log, Metadata, Record};

/// Number of `Summary {id} not valid at ...` debug records observed in the
/// most recent measurement window. Reset between queries.
static NON_MATCH_LOG_COUNT: AtomicUsize = AtomicUsize::new(0);
static LOGGER_INIT: Once = Once::new();

/// Drop-in `log` impl that just counts how many records contain the
/// "not valid at" sentinel that `summary_from_id_at_epoch` emits per non-match.
struct CountingLogger;

impl Log for CountingLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Debug
    }

    fn log(&self, record: &Record) {
        if record.level() <= Level::Debug
            && format!("{}", record.args()).contains("not valid at")
        {
            NON_MATCH_LOG_COUNT.fetch_add(1, Ordering::SeqCst);
        }
    }

    fn flush(&self) {}
}

fn install_counting_logger() {
    LOGGER_INIT.call_once(|| {
        log::set_logger(&CountingLogger).expect("logger already installed");
        log::set_max_level(LevelFilter::Debug);
    });
}

fn measured<F: FnOnce() -> R, R>(f: F) -> (R, usize) {
    NON_MATCH_LOG_COUNT.store(0, Ordering::SeqCst);
    let r = f();
    let n = NON_MATCH_LOG_COUNT.load(Ordering::SeqCst);
    (r, n)
}

#[test]
fn repeated_rotation_in_same_segment_should_hit_a_cache() {
    install_counting_logger();

    // Bundled by `download_test_data.sh`. Has 37 ITRF93 (NAIF id 3000)
    // segments starting 2025-01-01, each spanning ~1000 days (Chebyshev
    // triplet). Enough segments to span >1 DAF summary record.
    let bpc = BPC::load("../data/earth_2025_250826_2125_predict.bpc")
        .expect("bundled test BPC should load");
    let almanac = Almanac::from_bpc(bpc);

    // Two epochs that both fall inside the same segment
    // (2079-10-02 -> 2082-06-29). Picked deep enough into the file that
    // a linear scan from the start has to skip ~20 earlier segments before
    // finding the covering one.
    let ep_a = Epoch::from_gregorian_utc_at_midnight(2080, 6, 15);
    let ep_b = Epoch::from_gregorian_utc_at_midnight(2081, 6, 15);

    let (rot_a, misses_a) =
        measured(|| almanac.rotation_to_parent(EARTH_ITRF93, ep_a).unwrap());
    let (rot_b, misses_b) =
        measured(|| almanac.rotation_to_parent(EARTH_ITRF93, ep_b).unwrap());

    // Sanity: both rotations succeeded and refer to the same segment.
    assert!(rot_a.rot_mat[(0, 0)].is_finite());
    assert!(rot_b.rot_mat[(0, 0)].is_finite());

    eprintln!("first  query @ {ep_a}: {misses_a} 'not valid at' debug record(s)");
    eprintln!("second query @ {ep_b}: {misses_b} 'not valid at' debug record(s)");

    // Property 1: the linear scan really is happening. The covering segment
    // for 2080 is ~21st of 37, so we expect roughly 20 non-match records.
    assert!(
        misses_a > 1,
        "expected the linear scan to emit several 'not valid at' debug records \
         (one per skipped segment) on the first query, got {misses_a}. \
         If this is 0 or 1, the test setup or the file changed."
    );

    // Property 2: a 'last-found segment' cache, like SPICE keeps internally
    // for SPK/CK/PCK lookups, would let the second query short-circuit on the
    // cached segment with zero non-match records. Currently fails because
    // `summary_from_id_at_epoch` re-scans from the start every call.
    assert_eq!(
        misses_b, 0,
        "expected the second query (adjacent epoch in the same segment) to \
         hit a cached segment with zero non-match records, but got {misses_b} \
         -- identical to the first query's {misses_a}, indicating the linear \
         scan was repeated and no segment cache exists."
    );
}
