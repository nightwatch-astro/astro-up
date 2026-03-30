use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use astro_up_core::config::NetworkConfig;
use astro_up_core::download::{DownloadManager, DownloadRequest};

#[tokio::test]
async fn cancel_leaves_part_file_and_returns_cancelled() {
    let server = MockServer::start().await;

    // Slow response so we have time to cancel
    Mock::given(method("GET"))
        .and(path("/big.exe"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(vec![0u8; 1024 * 1024]) // 1MB
                .set_delay(std::time::Duration::from_secs(5)),
        )
        .mount(&server)
        .await;

    let config = NetworkConfig {
        user_agent: "astro-up-test/0.1".into(),
        ..NetworkConfig::default()
    };

    let (tx, _rx) = broadcast::channel(64);
    let manager = DownloadManager::new(&config, tx).unwrap();
    let dir = tempfile::tempdir().unwrap();

    let request = DownloadRequest {
        url: format!("{}/big.exe", server.uri()),
        expected_hash: None,
        dest_dir: dir.path().to_path_buf(),
        filename: "big.exe".into(),
        resume: false,
    };

    let cancel_token = CancellationToken::new();
    let cancel_clone = cancel_token.clone();

    // Cancel after a short delay
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        cancel_clone.cancel();
    });

    let err = manager.download(&request, cancel_token).await.unwrap_err();

    assert!(
        err.to_string().contains("cancelled"),
        "expected Cancelled error: {err}"
    );

    // .part file should still exist (left for resume)
    let part_path = dir.path().join("big.exe.part");
    assert!(
        part_path.exists(),
        ".part file should be preserved on cancel"
    );

    // Final file should NOT exist
    assert!(
        !dir.path().join("big.exe").exists(),
        "final file should not exist after cancel"
    );
}
