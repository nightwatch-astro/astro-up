use std::time::{Duration, SystemTime};

use astro_up_core::download::purge::purge;

/// Set a file's modified time to `days_ago` days in the past.
async fn backdate_file(path: &std::path::Path, days_ago: u64) {
    let past = SystemTime::now() - Duration::from_secs(days_ago * 86400);
    // Use filetime via std::fs::File::set_times (stabilized in Rust 1.75)
    let file = std::fs::File::options().write(true).open(path).unwrap();
    let ft = std::time::SystemTime::from(past).into();
    file.set_times(std::fs::FileTimes::new().set_modified(ft))
        .unwrap();
}

#[tokio::test]
async fn purge_deletes_old_files_keeps_recent() {
    let dir = tempfile::tempdir().unwrap();

    // Create files with different ages
    let old_file = dir.path().join("old-installer.exe");
    let recent_file = dir.path().join("recent-installer.exe");
    tokio::fs::write(&old_file, b"old content here").await.unwrap();
    tokio::fs::write(&recent_file, b"recent").await.unwrap();

    // Backdate old file to 45 days ago
    backdate_file(&old_file, 45).await;

    let result = purge(dir.path(), 30).await.unwrap();

    assert_eq!(result.files_deleted, 1);
    assert_eq!(result.bytes_reclaimed, b"old content here".len() as u64);
    assert!(!old_file.exists(), "old file should be deleted");
    assert!(recent_file.exists(), "recent file should be kept");
}

#[tokio::test]
async fn purge_disabled_when_zero_days() {
    let dir = tempfile::tempdir().unwrap();

    let old_file = dir.path().join("ancient.exe");
    tokio::fs::write(&old_file, b"data").await.unwrap();
    backdate_file(&old_file, 365).await;

    let result = purge(dir.path(), 0).await.unwrap();

    assert_eq!(result.files_deleted, 0);
    assert_eq!(result.bytes_reclaimed, 0);
    assert!(old_file.exists(), "file should NOT be deleted when purge disabled");
}

#[tokio::test]
async fn purge_skips_part_files() {
    let dir = tempfile::tempdir().unwrap();

    let part_file = dir.path().join("download.exe.part");
    let old_exe = dir.path().join("old.exe");
    tokio::fs::write(&part_file, b"partial download").await.unwrap();
    tokio::fs::write(&old_exe, b"old installer").await.unwrap();

    // Backdate both to 60 days
    backdate_file(&part_file, 60).await;
    backdate_file(&old_exe, 60).await;

    let result = purge(dir.path(), 30).await.unwrap();

    assert_eq!(result.files_deleted, 1);
    assert!(part_file.exists(), ".part file should NEVER be purged");
    assert!(!old_exe.exists(), "old .exe should be purged");
}

#[tokio::test]
async fn purge_empty_directory() {
    let dir = tempfile::tempdir().unwrap();

    let result = purge(dir.path(), 30).await.unwrap();

    assert_eq!(result.files_deleted, 0);
    assert_eq!(result.bytes_reclaimed, 0);
}
