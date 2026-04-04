//! Minisign signature verification.

use std::path::Path;

use crate::error::CoreError;

/// The production minisign public key, embedded at build time from minisign.pub.
pub const MINISIGN_PUBLIC_KEY: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/minisign.pub.key"));

/// Verify a catalog file against its minisign signature.
///
/// Reads both files into memory (catalog is <1MB) and verifies using
/// the embedded public key.
#[tracing::instrument(skip_all, fields(catalog = %catalog_path.display()))]
pub fn verify_catalog(catalog_path: &Path, sig_path: &Path) -> Result<(), CoreError> {
    verify_catalog_with_key(catalog_path, sig_path, MINISIGN_PUBLIC_KEY.trim())
}

/// Verify a catalog file against its minisign signature using the given public key.
///
/// Separated for testing with a test keypair.
pub fn verify_catalog_with_key(
    catalog_path: &Path,
    sig_path: &Path,
    public_key_b64: &str,
) -> Result<(), CoreError> {
    if !sig_path.exists() {
        return Err(CoreError::CatalogSignatureMissing);
    }

    let pk = minisign_verify::PublicKey::from_base64(public_key_b64).map_err(|e| {
        CoreError::CatalogFetchFailed {
            reason: format!("invalid public key: {e}"),
        }
    })?;

    let sig_contents = std::fs::read_to_string(sig_path)?;
    let sig = minisign_verify::Signature::decode(&sig_contents).map_err(|e| {
        CoreError::CatalogFetchFailed {
            reason: format!("invalid signature file: {e}"),
        }
    })?;

    let data = std::fs::read(catalog_path)?;

    pk.verify(&data, &sig, false)
        .map_err(|_| CoreError::CatalogSignatureInvalid)?;

    Ok(())
}

/// Verify catalog bytes and signature bytes in memory (no disk I/O).
///
/// Used by CatalogManager to verify before saving, preserving the previous catalog on failure.
pub fn verify_bytes(catalog_bytes: &[u8], sig_bytes: &[u8]) -> Result<(), CoreError> {
    verify_bytes_with_key(catalog_bytes, sig_bytes, MINISIGN_PUBLIC_KEY.trim())
}

/// Verify catalog bytes and signature bytes using the given public key.
pub fn verify_bytes_with_key(
    catalog_bytes: &[u8],
    sig_bytes: &[u8],
    public_key_b64: &str,
) -> Result<(), CoreError> {
    let pk = minisign_verify::PublicKey::from_base64(public_key_b64).map_err(|e| {
        CoreError::CatalogFetchFailed {
            reason: format!("invalid public key: {e}"),
        }
    })?;

    let sig_str = std::str::from_utf8(sig_bytes).map_err(|_| CoreError::CatalogSignatureInvalid)?;
    let sig = minisign_verify::Signature::decode(sig_str)
        .map_err(|_| CoreError::CatalogSignatureInvalid)?;

    pk.verify(catalog_bytes, &sig, false)
        .map_err(|_| CoreError::CatalogSignatureInvalid)?;

    Ok(())
}

/// Build the signature file path from a catalog path (e.g., `catalog.db` → `catalog.db.minisig`).
pub fn sig_path_for(catalog_path: &Path) -> std::path::PathBuf {
    let mut p = catalog_path.as_os_str().to_owned();
    p.push(".minisig");
    std::path::PathBuf::from(p)
}
