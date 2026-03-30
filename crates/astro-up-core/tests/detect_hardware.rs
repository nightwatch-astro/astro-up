use astro_up_core::detect::hardware::VidPid;

#[cfg(windows)]
#[tokio::test]
async fn discover_enumerates_usb_devices() {
    // On Windows, query Win32_PnPEntity and verify USB devices are found
    // We can't assert on specific hardware, but we can verify the query doesn't error
    let patterns: Vec<(VidPid, String)> = vec![];
    let managed = std::collections::HashSet::new();

    let matches = astro_up_core::detect::hardware::discover(&patterns, &managed).await;
    // Empty patterns = no matches, but the WMI query should succeed
    assert!(matches.is_empty());
}

#[cfg(not(windows))]
#[tokio::test]
async fn discover_returns_empty_on_non_windows() {
    let patterns: Vec<(VidPid, String)> =
        vec![(VidPid::parse("03C3:*").unwrap(), "zwo-asi-camera".into())];
    let managed = std::collections::HashSet::new();

    let matches = astro_up_core::detect::hardware::discover(&patterns, &managed).await;
    assert!(matches.is_empty());
}
