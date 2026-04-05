#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::match_wildcard_for_single_variants
)]

use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

use astro_up_core::config::NetworkConfig;
use astro_up_core::download::{DownloadManager, DownloadRequest, DownloadResult};

fn test_config() -> NetworkConfig {
    NetworkConfig {
        user_agent: "astro-up-test/0.1".into(),
        ..NetworkConfig::default()
    }
}

#[tokio::test]
async fn resume_with_range_206() {
    let server = MockServer::start().await;
    let full_body = b"AAAAABBBBB"; // 10 bytes: 5 + 5

    // Dynamic response: if Range header present, return 206 with remaining bytes
    Mock::given(method("GET"))
        .and(path("/resume.exe"))
        .respond_with(|req: &Request| {
            if let Some(range) = req.headers.get("Range") {
                let range_str = range.to_str().unwrap();
                // Parse "bytes=5-"
                if let Some(start_str) = range_str.strip_prefix("bytes=") {
                    let start_str = start_str.trim_end_matches('-');
                    if let Ok(start) = start_str.parse::<usize>() {
                        let remaining = &b"AAAAABBBBB"[start..];
                        return ResponseTemplate::new(206)
                            .set_body_bytes(remaining.to_vec())
                            .insert_header("Content-Range", format!("bytes {start}-9/10"));
                    }
                }
            }
            // No Range header: full response
            ResponseTemplate::new(200).set_body_bytes(b"AAAAABBBBB".to_vec())
        })
        .mount(&server)
        .await;

    let (tx, _rx) = broadcast::channel(64);
    let manager = DownloadManager::new(&test_config(), tx).unwrap();
    let dir = tempfile::tempdir().unwrap();

    // Create a .part file with first 5 bytes
    let part_path = dir.path().join("resume.exe.part");
    tokio::fs::write(&part_path, &full_body[..5]).await.unwrap();

    let request = DownloadRequest {
        url: format!("{}/resume.exe", server.uri()),
        expected_hash: None,
        dest_dir: dir.path().to_path_buf(),
        filename: "resume.exe".into(),
        resume: true,
    };

    let result = manager
        .download(&request, CancellationToken::new())
        .await
        .unwrap();

    match result {
        DownloadResult::Success {
            path,
            bytes_downloaded,
            resumed,
            ..
        } => {
            assert!(resumed, "should be marked as resumed");
            assert_eq!(bytes_downloaded, 10);
            let contents = tokio::fs::read(&path).await.unwrap();
            assert_eq!(contents, full_body);
        }
        _ => panic!("expected Success"),
    }
}

#[tokio::test]
async fn resume_server_returns_200_restarts_from_scratch() {
    let server = MockServer::start().await;
    let full_body = b"complete file content";

    // Server always returns 200 (no Range support)
    Mock::given(method("GET"))
        .and(path("/no-range.exe"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(full_body.to_vec()))
        .mount(&server)
        .await;

    let (tx, _rx) = broadcast::channel(64);
    let manager = DownloadManager::new(&test_config(), tx).unwrap();
    let dir = tempfile::tempdir().unwrap();

    // Create a .part file with partial data
    let part_path = dir.path().join("no-range.exe.part");
    tokio::fs::write(&part_path, b"partial").await.unwrap();

    let request = DownloadRequest {
        url: format!("{}/no-range.exe", server.uri()),
        expected_hash: None,
        dest_dir: dir.path().to_path_buf(),
        filename: "no-range.exe".into(),
        resume: true,
    };

    let result = manager
        .download(&request, CancellationToken::new())
        .await
        .unwrap();

    match result {
        DownloadResult::Success {
            path,
            resumed,
            bytes_downloaded,
            ..
        } => {
            // Server returned 200, so it's a full re-download, not a resume
            assert!(
                !resumed,
                "should NOT be marked as resumed (server returned 200)"
            );
            assert_eq!(bytes_downloaded, full_body.len() as u64);
            let contents = tokio::fs::read(&path).await.unwrap();
            assert_eq!(contents, full_body);
        }
        _ => panic!("expected Success"),
    }
}

#[tokio::test]
async fn resume_disabled_ignores_part_file() {
    let server = MockServer::start().await;
    let full_body = b"fresh download";

    Mock::given(method("GET"))
        .and(path("/fresh.exe"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(full_body.to_vec()))
        .mount(&server)
        .await;

    let (tx, _rx) = broadcast::channel(64);
    let manager = DownloadManager::new(&test_config(), tx).unwrap();
    let dir = tempfile::tempdir().unwrap();

    // Create a .part file
    tokio::fs::write(dir.path().join("fresh.exe.part"), b"old partial")
        .await
        .unwrap();

    let request = DownloadRequest {
        url: format!("{}/fresh.exe", server.uri()),
        expected_hash: None,
        dest_dir: dir.path().to_path_buf(),
        filename: "fresh.exe".into(),
        resume: false, // resume disabled
    };

    let result = manager
        .download(&request, CancellationToken::new())
        .await
        .unwrap();

    match result {
        DownloadResult::Success {
            path,
            resumed,
            bytes_downloaded,
            ..
        } => {
            assert!(!resumed);
            assert_eq!(bytes_downloaded, full_body.len() as u64);
            let contents = tokio::fs::read(&path).await.unwrap();
            assert_eq!(contents, full_body);
        }
        _ => panic!("expected Success"),
    }
}

#[tokio::test]
async fn resume_restarts_when_server_file_is_newer() {
    let server = MockServer::start().await;
    let full_body = b"updated file content";

    // Server responds to Range request with 206 but a newer Last-Modified
    Mock::given(method("GET"))
        .and(path("/newer.exe"))
        .respond_with(|req: &Request| {
            if req.headers.get("Range").is_some() {
                ResponseTemplate::new(206)
                    .set_body_bytes(b"partial".to_vec())
                    .insert_header("Last-Modified", "Sun, 01 Jan 2090 00:00:00 GMT")
                    .insert_header("Content-Range", "bytes 5-19/20")
            } else {
                ResponseTemplate::new(200).set_body_bytes(b"updated file content".to_vec())
            }
        })
        .mount(&server)
        .await;

    let (tx, _rx) = broadcast::channel(64);
    let manager = DownloadManager::new(&test_config(), tx).unwrap();
    let dir = tempfile::tempdir().unwrap();

    // Create a .part file with old content, backdated
    let part_path = dir.path().join("newer.exe.part");
    tokio::fs::write(&part_path, b"old p").await.unwrap();
    let past = std::time::SystemTime::now() - std::time::Duration::from_secs(365 * 86400);
    let file = std::fs::File::options()
        .write(true)
        .open(&part_path)
        .unwrap();
    file.set_times(std::fs::FileTimes::new().set_modified(past))
        .unwrap();
    drop(file);

    let request = DownloadRequest {
        url: format!("{}/newer.exe", server.uri()),
        expected_hash: None,
        dest_dir: dir.path().to_path_buf(),
        filename: "newer.exe".into(),
        resume: true,
    };

    let result = manager
        .download(&request, CancellationToken::new())
        .await
        .unwrap();

    match result {
        DownloadResult::Success {
            path,
            resumed,
            bytes_downloaded,
            ..
        } => {
            assert!(!resumed, "should NOT be resumed when server file is newer");
            assert_eq!(bytes_downloaded, full_body.len() as u64);
            let contents = tokio::fs::read(&path).await.unwrap();
            assert_eq!(contents, full_body);
        }
        _ => panic!("expected Success"),
    }
}
