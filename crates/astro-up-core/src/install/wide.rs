/// Converts a string to a null-terminated wide (UTF-16) string for Windows APIs.
#[cfg(windows)]
pub fn to_wide_null(s: &str) -> Vec<u16> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}
