use std::time::Instant;

use astro_up_core::detect::pe;

/// SC-001 proxy: verify PE detection completes quickly (cross-platform component).
/// Full scan benchmark requires Windows + real catalog — this validates the
/// per-package detection path is fast enough.
#[test]
fn pe_detection_completes_under_100ms() {
    let start = Instant::now();
    // Run PE detection 95 times (simulating full catalog scan)
    for _ in 0..95 {
        let _ = pe::read_pe_version_sync("tests/fixtures/test.exe");
    }
    let elapsed = start.elapsed();

    // 95 PE reads should complete well under 5 seconds
    assert!(
        elapsed.as_secs() < 5,
        "95 PE detections took {elapsed:?}, expected <5s (SC-001)"
    );

    // Each individual read should be <50ms
    let per_read = elapsed / 95;
    assert!(
        per_read.as_millis() < 50,
        "per-read {per_read:?}, expected <50ms"
    );
}
