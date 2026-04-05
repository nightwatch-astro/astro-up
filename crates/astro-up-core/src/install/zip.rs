use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use crate::error::CoreError;

/// Extracts a ZIP archive to `dest_dir` with zip-slip protection and single-root flattening.
///
/// Returns the directory where files were extracted.
///
/// # Zip-slip protection
/// Every entry path is validated via `enclosed_name()`. Entries with `..` components
/// or absolute paths are rejected.
///
/// # Single-root flattening
/// If all entries share a single common root directory (e.g., `NINA-3.1/`), that prefix
/// is stripped to avoid double nesting like `dest/NINA-3.1/NINA-3.1/`.
pub async fn extract_zip(archive_path: &Path, dest_dir: &Path) -> Result<PathBuf, CoreError> {
    let archive_path = archive_path.to_path_buf();
    let dest_dir = dest_dir.to_path_buf();

    tokio::task::spawn_blocking(move || extract_zip_sync(&archive_path, &dest_dir))
        .await
        .map_err(|e| CoreError::Io(io::Error::other(e)))?
}

fn extract_zip_sync(archive_path: &Path, dest_dir: &Path) -> Result<PathBuf, CoreError> {
    let file = fs::File::open(archive_path)?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| CoreError::Io(io::Error::new(io::ErrorKind::InvalidData, e)))?;

    if archive.is_empty() {
        fs::create_dir_all(dest_dir)?;
        return Ok(dest_dir.to_path_buf());
    }

    // Detect single-root directory
    let strip_prefix = detect_single_root(&mut archive)?;

    fs::create_dir_all(dest_dir)?;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| CoreError::Io(io::Error::new(io::ErrorKind::InvalidData, e)))?;

        // Zip-slip protection: enclosed_name() returns None for unsafe paths
        let safe_path = entry.enclosed_name().ok_or_else(|| {
            CoreError::Io(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "zip-slip: rejected unsafe path in archive: {}",
                    entry.name()
                ),
            ))
        })?;

        // Strip the common root prefix if single-root detected
        let relative = if let Some(prefix) = &strip_prefix {
            safe_path
                .strip_prefix(prefix)
                .unwrap_or(&safe_path)
                .to_path_buf()
        } else {
            safe_path.to_path_buf()
        };

        // Skip empty relative paths (the root directory entry itself)
        if relative.as_os_str().is_empty() {
            continue;
        }

        let target = dest_dir.join(&relative);

        if entry.is_dir() {
            fs::create_dir_all(&target)?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut outfile = fs::File::create(&target)?;
            io::copy(&mut entry, &mut outfile)?;
        }
    }

    Ok(dest_dir.to_path_buf())
}

/// Detects if all entries share a single common root directory.
///
/// Returns `Some(prefix)` if a single root is detected, `None` otherwise.
fn detect_single_root(
    archive: &mut zip::ZipArchive<fs::File>,
) -> Result<Option<PathBuf>, CoreError> {
    let mut roots = HashSet::new();
    let mut has_root_files = false;

    for i in 0..archive.len() {
        let entry = archive
            .by_index(i)
            .map_err(|e| CoreError::Io(io::Error::new(io::ErrorKind::InvalidData, e)))?;

        let safe_path = match entry.enclosed_name() {
            Some(p) => p.to_path_buf(),
            None => continue, // Will be caught during extraction
        };

        // Get the first component
        let mut components = safe_path.components();
        if let Some(Component::Normal(first)) = components.next() {
            if components.next().is_some() {
                // Has at least two components — record the root
                roots.insert(first.to_os_string());
            } else if !entry.is_dir() {
                // File at the root level — no single root
                has_root_files = true;
            } else {
                // Directory at root level
                roots.insert(first.to_os_string());
            }
        }
    }

    if roots.len() == 1 && !has_root_files {
        Ok(roots.into_iter().next().map(PathBuf::from))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::unreadable_literal)]
mod tests {
    use super::*;
    use std::io::Write;

    fn create_test_zip(entries: &[(&str, &[u8])]) -> tempfile::NamedTempFile {
        let file = tempfile::NamedTempFile::new().unwrap();
        let mut zip = zip::ZipWriter::new(file.reopen().unwrap());
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);

        for (name, content) in entries {
            if name.ends_with('/') {
                zip.add_directory(*name, options).unwrap();
            } else {
                zip.start_file(*name, options).unwrap();
                zip.write_all(content).unwrap();
            }
        }
        zip.finish().unwrap();
        file
    }

    #[tokio::test]
    async fn extract_normal_archive() {
        let zip_file =
            create_test_zip(&[("readme.txt", b"hello"), ("src/main.rs", b"fn main() {}")]);
        let dest = tempfile::tempdir().unwrap();

        let result = extract_zip(zip_file.path(), dest.path()).await;
        assert!(result.is_ok());
        assert!(dest.path().join("readme.txt").exists());
        assert!(dest.path().join("src/main.rs").exists());
    }

    #[tokio::test]
    async fn extract_single_root_flattened() {
        let zip_file = create_test_zip(&[
            ("NINA-3.1/", &[]),
            ("NINA-3.1/nina.exe", b"exe"),
            ("NINA-3.1/lib/core.dll", b"dll"),
        ]);
        let dest = tempfile::tempdir().unwrap();

        let result = extract_zip(zip_file.path(), dest.path()).await;
        assert!(result.is_ok());
        // Single root stripped — files extracted directly
        assert!(dest.path().join("nina.exe").exists());
        assert!(dest.path().join("lib/core.dll").exists());
        // Should NOT have the nested directory
        assert!(!dest.path().join("NINA-3.1").exists());
    }

    #[tokio::test]
    async fn extract_multi_root_not_flattened() {
        let zip_file = create_test_zip(&[
            ("bin/app.exe", b"exe"),
            ("lib/core.dll", b"dll"),
            ("readme.txt", b"hello"),
        ]);
        let dest = tempfile::tempdir().unwrap();

        let result = extract_zip(zip_file.path(), dest.path()).await;
        assert!(result.is_ok());
        assert!(dest.path().join("bin/app.exe").exists());
        assert!(dest.path().join("lib/core.dll").exists());
        assert!(dest.path().join("readme.txt").exists());
    }

    #[tokio::test]
    async fn reject_zip_slip_attack() {
        // Build a raw ZIP file with a malicious "../evil.txt" entry.
        // The zip crate's ZipWriter sanitizes paths, so we construct raw bytes.
        // ZIP local file header format: signature + fields + filename + data
        let malicious_name = b"../evil.txt";
        let payload = b"malicious content";
        let crc = 0u32; // CRC validation not needed — we're testing path rejection

        // Local file header
        let mut local_header = Vec::new();
        local_header.extend_from_slice(&0x04034b50u32.to_le_bytes()); // signature
        local_header.extend_from_slice(&20u16.to_le_bytes()); // version needed
        local_header.extend_from_slice(&0u16.to_le_bytes()); // flags
        local_header.extend_from_slice(&0u16.to_le_bytes()); // compression (stored)
        local_header.extend_from_slice(&0u16.to_le_bytes()); // mod time
        local_header.extend_from_slice(&0u16.to_le_bytes()); // mod date
        local_header.extend_from_slice(&crc.to_le_bytes()); // crc32
        local_header.extend_from_slice(&(payload.len() as u32).to_le_bytes()); // compressed
        local_header.extend_from_slice(&(payload.len() as u32).to_le_bytes()); // uncompressed
        local_header.extend_from_slice(&(malicious_name.len() as u16).to_le_bytes()); // name len
        local_header.extend_from_slice(&0u16.to_le_bytes()); // extra len

        let local_offset = 0u32;

        // Central directory header
        let mut central = Vec::new();
        central.extend_from_slice(&0x02014b50u32.to_le_bytes()); // signature
        central.extend_from_slice(&20u16.to_le_bytes()); // version made by
        central.extend_from_slice(&20u16.to_le_bytes()); // version needed
        central.extend_from_slice(&0u16.to_le_bytes()); // flags
        central.extend_from_slice(&0u16.to_le_bytes()); // compression
        central.extend_from_slice(&0u16.to_le_bytes()); // mod time
        central.extend_from_slice(&0u16.to_le_bytes()); // mod date
        central.extend_from_slice(&crc.to_le_bytes()); // crc32
        central.extend_from_slice(&(payload.len() as u32).to_le_bytes()); // compressed
        central.extend_from_slice(&(payload.len() as u32).to_le_bytes()); // uncompressed
        central.extend_from_slice(&(malicious_name.len() as u16).to_le_bytes()); // name len
        central.extend_from_slice(&0u16.to_le_bytes()); // extra len
        central.extend_from_slice(&0u16.to_le_bytes()); // comment len
        central.extend_from_slice(&0u16.to_le_bytes()); // disk number
        central.extend_from_slice(&0u16.to_le_bytes()); // internal attrs
        central.extend_from_slice(&0u32.to_le_bytes()); // external attrs
        central.extend_from_slice(&local_offset.to_le_bytes()); // local header offset

        // Assemble the ZIP file
        let mut zip_bytes = Vec::new();
        zip_bytes.extend_from_slice(&local_header);
        zip_bytes.extend_from_slice(malicious_name);
        zip_bytes.extend_from_slice(payload);

        let central_offset = zip_bytes.len() as u32;
        zip_bytes.extend_from_slice(&central);
        zip_bytes.extend_from_slice(malicious_name);

        let central_size = (zip_bytes.len() as u32) - central_offset;

        // End of central directory
        zip_bytes.extend_from_slice(&0x06054b50u32.to_le_bytes()); // signature
        zip_bytes.extend_from_slice(&0u16.to_le_bytes()); // disk number
        zip_bytes.extend_from_slice(&0u16.to_le_bytes()); // central dir disk
        zip_bytes.extend_from_slice(&1u16.to_le_bytes()); // entries on disk
        zip_bytes.extend_from_slice(&1u16.to_le_bytes()); // total entries
        zip_bytes.extend_from_slice(&central_size.to_le_bytes()); // central dir size
        zip_bytes.extend_from_slice(&central_offset.to_le_bytes()); // central dir offset
        zip_bytes.extend_from_slice(&0u16.to_le_bytes()); // comment len

        let file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(file.path(), &zip_bytes).unwrap();

        let dest = tempfile::tempdir().unwrap();
        let result = extract_zip(file.path(), dest.path()).await;

        // Must reject the traversal path
        assert!(result.is_err(), "zip-slip path should be rejected");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("zip-slip"),
            "error should mention zip-slip: {err}"
        );
        // Verify no file was written outside dest
        assert!(
            !dest.path().join("../evil.txt").exists(),
            "malicious file must not be created"
        );
    }

    #[tokio::test]
    async fn extract_empty_archive() {
        let file = tempfile::NamedTempFile::new().unwrap();
        let zip = zip::ZipWriter::new(file.reopen().unwrap());
        zip.finish().unwrap();

        let dest = tempfile::tempdir().unwrap();
        let result = extract_zip(file.path(), dest.path()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn files_only_no_dirs_no_flattening() {
        let zip_file = create_test_zip(&[("file1.txt", b"one"), ("file2.txt", b"two")]);
        let dest = tempfile::tempdir().unwrap();

        let result = extract_zip(zip_file.path(), dest.path()).await;
        assert!(result.is_ok());
        assert!(dest.path().join("file1.txt").exists());
        assert!(dest.path().join("file2.txt").exists());
    }
}
