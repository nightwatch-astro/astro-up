use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use astro_up_core::config::NetworkConfig;
use astro_up_core::download::{DownloadManager, DownloadRequest, DownloadResult};
use astro_up_core::events::Event;

fn test_config(_server_uri: &str) -> NetworkConfig {
    NetworkConfig {
        user_agent: format!("astro-up-test/{}", env!("CARGO_PKG_VERSION")),
        ..NetworkConfig::default()
    }
}

#[tokio::test]
async fn download_basic_file() {
    let server = MockServer::start().await;
    let body = b"hello world download test content";

    Mock::given(method("GET"))
        .and(path("/test-file.exe"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(body.to_vec()))
        .mount(&server)
        .await;

    let (event_tx, mut event_rx) = broadcast::channel(64);
    let config = test_config(&server.uri());
    let manager = DownloadManager::new(&config, event_tx).unwrap();

    let dir = tempfile::tempdir().unwrap();
    let request = DownloadRequest {
        url: format!("{}/test-file.exe", server.uri()),
        expected_hash: None,
        dest_dir: dir.path().to_path_buf(),
        filename: "test-file.exe".into(),
        resume: false,
    };

    let result = manager
        .download(&request, CancellationToken::new())
        .await
        .unwrap();

    // Verify result
    match &result {
        DownloadResult::Success {
            path,
            hash_verified,
            bytes_downloaded,
            resumed,
        } => {
            assert_eq!(*path, dir.path().join("test-file.exe"));
            assert!(!hash_verified);
            assert_eq!(*bytes_downloaded, body.len() as u64);
            assert!(!resumed);
        }
        DownloadResult::Cached { .. } => panic!("expected Success, got Cached"),
    }

    // Verify file contents
    let contents = tokio::fs::read(dir.path().join("test-file.exe"))
        .await
        .unwrap();
    assert_eq!(contents, body);

    // Verify .part file was cleaned up
    assert!(!dir.path().join("test-file.exe.part").exists());

    // Verify events were emitted
    let mut got_started = false;
    let mut got_complete = false;
    // Drain all events
    loop {
        match event_rx.try_recv() {
            Ok(Event::DownloadStarted { id, url }) => {
                assert_eq!(id, "test-file.exe");
                assert!(url.contains("/test-file.exe"));
                got_started = true;
            }
            Ok(Event::DownloadComplete { id }) => {
                assert_eq!(id, "test-file.exe");
                got_complete = true;
            }
            Ok(Event::DownloadProgress { .. }) => {}
            Ok(_) => {}
            Err(_) => break,
        }
    }
    assert!(got_started, "expected DownloadStarted event");
    assert!(got_complete, "expected DownloadComplete event");
}

#[tokio::test]
async fn download_error_reports_url_and_status() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/forbidden.exe"))
        .respond_with(ResponseTemplate::new(403))
        .mount(&server)
        .await;

    let (event_tx, _rx) = broadcast::channel(64);
    let config = test_config(&server.uri());
    let manager = DownloadManager::new(&config, event_tx).unwrap();

    let dir = tempfile::tempdir().unwrap();
    let request = DownloadRequest {
        url: format!("{}/forbidden.exe", server.uri()),
        expected_hash: None,
        dest_dir: dir.path().to_path_buf(),
        filename: "forbidden.exe".into(),
        resume: false,
    };

    let err = manager
        .download(&request, CancellationToken::new())
        .await
        .unwrap_err();

    let msg = err.to_string();
    assert!(msg.contains("/forbidden.exe"), "error should contain URL: {msg}");
    assert!(msg.contains("403"), "error should contain status code: {msg}");
}

#[tokio::test]
async fn download_sequential_lock_rejects_concurrent() {
    let server = MockServer::start().await;

    // Slow response to hold the lock
    Mock::given(method("GET"))
        .and(path("/slow.exe"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(vec![0u8; 1024])
                .set_delay(std::time::Duration::from_secs(2)),
        )
        .mount(&server)
        .await;

    let (event_tx, _rx) = broadcast::channel(64);
    let config = test_config(&server.uri());
    let manager = std::sync::Arc::new(DownloadManager::new(&config, event_tx).unwrap());

    let dir = tempfile::tempdir().unwrap();
    let dir_path = dir.path().to_path_buf();

    // Start first download in background
    let m1 = manager.clone();
    let d1 = dir_path.clone();
    let handle = tokio::spawn(async move {
        let req = DownloadRequest {
            url: format!("{}/slow.exe", server.uri()),
            expected_hash: None,
            dest_dir: d1,
            filename: "slow.exe".into(),
            resume: false,
        };
        m1.download(&req, CancellationToken::new()).await
    });

    // Give first download time to acquire lock
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Second download should be rejected
    let req2 = DownloadRequest {
        url: "http://example.com/other.exe".into(),
        expected_hash: None,
        dest_dir: dir_path,
        filename: "other.exe".into(),
        resume: false,
    };
    let err = manager
        .download(&req2, CancellationToken::new())
        .await
        .unwrap_err();

    assert!(
        err.to_string().contains("already in progress"),
        "expected DownloadInProgress error: {err}"
    );

    // Clean up
    let _ = handle.await;
}

#[tokio::test]
async fn download_creates_dest_dir() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/file.exe"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"data".to_vec()))
        .mount(&server)
        .await;

    let (event_tx, _rx) = broadcast::channel(64);
    let config = test_config(&server.uri());
    let manager = DownloadManager::new(&config, event_tx).unwrap();

    let dir = tempfile::tempdir().unwrap();
    let nested = dir.path().join("deep").join("nested").join("dir");

    let request = DownloadRequest {
        url: format!("{}/file.exe", server.uri()),
        expected_hash: None,
        dest_dir: nested.clone(),
        filename: "file.exe".into(),
        resume: false,
    };

    let result = manager
        .download(&request, CancellationToken::new())
        .await
        .unwrap();

    match result {
        DownloadResult::Success { path, .. } => {
            assert!(path.exists());
            assert_eq!(path, nested.join("file.exe"));
        }
        _ => panic!("expected Success"),
    }
}
