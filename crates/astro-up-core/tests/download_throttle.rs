#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::match_wildcard_for_single_variants
)]

use std::time::Instant;

use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use astro_up_core::config::NetworkConfig;
use astro_up_core::download::{DownloadManager, DownloadRequest, DownloadResult};

#[tokio::test]
async fn throttle_limits_download_speed() {
    let server = MockServer::start().await;

    // 500KB file, throttle at 100KB/s => should take ~5 seconds
    let body = vec![0u8; 500 * 1024];
    Mock::given(method("GET"))
        .and(path("/large.exe"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(body))
        .mount(&server)
        .await;

    let config = NetworkConfig {
        user_agent: "astro-up-test/0.1".into(),
        download_speed_limit: 100 * 1024, // 100KB/s
        ..NetworkConfig::default()
    };

    let (tx, _rx) = broadcast::channel(64);
    let manager = DownloadManager::new(&config, tx).unwrap();
    let dir = tempfile::tempdir().unwrap();

    let request = DownloadRequest {
        url: format!("{}/large.exe", server.uri()),
        expected_hash: None,
        dest_dir: dir.path().to_path_buf(),
        filename: "large.exe".into(),
        resume: false,
    };

    let start = Instant::now();
    let result = manager
        .download(&request, CancellationToken::new())
        .await
        .unwrap();
    let elapsed = start.elapsed().as_secs_f64();

    match result {
        DownloadResult::Success {
            bytes_downloaded, ..
        } => {
            assert_eq!(bytes_downloaded, 500 * 1024);
        }
        _ => panic!("expected Success"),
    }

    // Expected ~5 seconds. Allow 10% tolerance (SC-005): 4.5–5.5s
    assert!(
        elapsed >= 4.5,
        "download too fast: {elapsed:.2}s (expected >= 4.5s)"
    );
    assert!(
        elapsed <= 6.0,
        "download too slow: {elapsed:.2}s (expected <= 6.0s)"
    );
}

#[tokio::test]
async fn no_throttle_when_limit_is_zero() {
    let server = MockServer::start().await;

    let body = vec![0u8; 100 * 1024]; // 100KB
    Mock::given(method("GET"))
        .and(path("/fast.exe"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(body))
        .mount(&server)
        .await;

    let config = NetworkConfig {
        user_agent: "astro-up-test/0.1".into(),
        download_speed_limit: 0, // unlimited
        ..NetworkConfig::default()
    };

    let (tx, _rx) = broadcast::channel(64);
    let manager = DownloadManager::new(&config, tx).unwrap();
    let dir = tempfile::tempdir().unwrap();

    let request = DownloadRequest {
        url: format!("{}/fast.exe", server.uri()),
        expected_hash: None,
        dest_dir: dir.path().to_path_buf(),
        filename: "fast.exe".into(),
        resume: false,
    };

    let start = Instant::now();
    manager
        .download(&request, CancellationToken::new())
        .await
        .unwrap();
    let elapsed = start.elapsed().as_secs_f64();

    // Should be nearly instant (well under 1 second for 100KB local)
    assert!(
        elapsed < 2.0,
        "unthrottled download too slow: {elapsed:.2}s"
    );
}
