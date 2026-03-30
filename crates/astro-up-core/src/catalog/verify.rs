//! Minisign signature verification.

use std::path::Path;

use crate::error::CoreError;

/// The production minisign public key, embedded at compile time.
pub const MINISIGN_PUBLIC_KEY: &str = "RWT3Z/NUV2mo2nf2YDHF/Iyz9NFR7+gkUHa0rTlAcIBBxg+eqG3LUItj";

/// Verify a catalog file against its minisign signature.
///
/// Reads both files into memory (catalog is <1MB) and verifies using
/// the embedded public key.
#[tracing::instrument(skip_all, fields(catalog = %catalog_path.display()))]
pub fn verify_catalog(catalog_path: &Path, sig_path: &Path) -> Result<(), CoreError> {
    verify_catalog_with_key(catalog_path, sig_path, MINISIGN_PUBLIC_KEY)
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

/// Build the signature file path from a catalog path (e.g., `catalog.db` → `catalog.db.minisig`).
pub fn sig_path_for(catalog_path: &Path) -> std::path::PathBuf {
    let mut p = catalog_path.as_os_str().to_owned();
    p.push(".minisig");
    std::path::PathBuf::from(p)
}
