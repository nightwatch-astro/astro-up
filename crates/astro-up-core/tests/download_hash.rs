#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::match_wildcard_for_single_variants
)]

use sha2::{Digest, Sha256};
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use astro_up_core::config::NetworkConfig;
use astro_up_core::download::{DownloadManager, DownloadRequest, DownloadResult};

fn test_config() -> NetworkConfig {
    NetworkConfig {
        user_agent: "astro-up-test/0.1".into(),
        ..NetworkConfig::default()
    }
}

#[tokio::test]
async fn download_with_correct_hash() {
    let server = MockServer::start().await;
    let body = b"verified content here";
    let digest = Sha256::digest(body);
    let expected_hash: String = astro_up_core::hex_encode(&digest);

    Mock::given(method("GET"))
        .and(path("/verified.exe"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(body.to_vec()))
        .mount(&server)
        .await;

    let (tx, _rx) = broadcast::channel(64);
    let manager = DownloadManager::new(&test_config(), tx).unwrap();
    let dir = tempfile::tempdir().unwrap();

    let request = DownloadRequest {
        url: format!("{}/verified.exe", server.uri()),
        expected_hash: Some(expected_hash),
        dest_dir: dir.path().to_path_buf(),
        filename: "verified.exe".into(),
        resume: false,
    };

    let result = manager
        .download(&request, CancellationToken::new())
        .await
        .unwrap();

    match result {
        DownloadResult::Success {
            hash_verified,
            path,
            ..
        } => {
            assert!(hash_verified);
            assert!(path.exists());
        }
        _ => panic!("expected Success"),
    }
}

#[tokio::test]
async fn download_with_wrong_hash_fails() {
    let server = MockServer::start().await;
    let body = b"some content";

    Mock::given(method("GET"))
        .and(path("/bad-hash.exe"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(body.to_vec()))
        .mount(&server)
        .await;

    let (tx, _rx) = broadcast::channel(64);
    let manager = DownloadManager::new(&test_config(), tx).unwrap();
    let dir = tempfile::tempdir().unwrap();

    let request = DownloadRequest {
        url: format!("{}/bad-hash.exe", server.uri()),
        expected_hash: Some(
            "0000000000000000000000000000000000000000000000000000000000000000".into(),
        ),
        dest_dir: dir.path().to_path_buf(),
        filename: "bad-hash.exe".into(),
        resume: false,
    };

    let err = manager
        .download(&request, CancellationToken::new())
        .await
        .unwrap_err();

    assert!(
        err.to_string().contains("checksum mismatch"),
        "expected ChecksumMismatch: {err}"
    );

    // .part file should be deleted
    assert!(!dir.path().join("bad-hash.exe.part").exists());
    // Final file should not exist
    assert!(!dir.path().join("bad-hash.exe").exists());
}

#[tokio::test]
async fn download_without_expected_hash() {
    let server = MockServer::start().await;
    let body = b"no hash check";

    Mock::given(method("GET"))
        .and(path("/no-hash.exe"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(body.to_vec()))
        .mount(&server)
        .await;

    let (tx, _rx) = broadcast::channel(64);
    let manager = DownloadManager::new(&test_config(), tx).unwrap();
    let dir = tempfile::tempdir().unwrap();

    let request = DownloadRequest {
        url: format!("{}/no-hash.exe", server.uri()),
        expected_hash: None,
        dest_dir: dir.path().to_path_buf(),
        filename: "no-hash.exe".into(),
        resume: false,
    };

    let result = manager
        .download(&request, CancellationToken::new())
        .await
        .unwrap();

    match result {
        DownloadResult::Success {
            hash_verified,
            path,
            ..
        } => {
            assert!(!hash_verified);
            assert!(path.exists());
        }
        _ => panic!("expected Success"),
    }
}

#[tokio::test]
async fn download_returns_cached_on_304() {
    let server = MockServer::start().await;
    let body = b"original content";

    // First request: normal download
    Mock::given(method("GET"))
        .and(path("/cached.exe"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(body.to_vec()))
        .expect(1)
        .mount(&server)
        .await;

    let (tx, _rx) = broadcast::channel(64);
    let manager = DownloadManager::new(&test_config(), tx).unwrap();
    let dir = tempfile::tempdir().unwrap();

    let request = DownloadRequest {
        url: format!("{}/cached.exe", server.uri()),
        expected_hash: None,
        dest_dir: dir.path().to_path_buf(),
        filename: "cached.exe".into(),
        resume: false,
    };

    // First download
    manager
        .download(&request, CancellationToken::new())
        .await
        .unwrap();

    // Now mount a 304 for HEAD requests (conditional)
    server.reset().await;
    Mock::given(method("HEAD"))
        .and(path("/cached.exe"))
        .respond_with(ResponseTemplate::new(304))
        .mount(&server)
        .await;

    // Second download should return Cached
    let result = manager
        .download(&request, CancellationToken::new())
        .await
        .unwrap();

    match result {
        DownloadResult::Cached { path } => {
            assert_eq!(path, dir.path().join("cached.exe"));
        }
        _ => panic!("expected Cached, got {result:?}"),
    }
}
