use reqwest::redirect;

use crate::config::NetworkConfig;
use crate::error::CoreError;

/// Build a configured reqwest client from network settings.
pub fn build_client(config: &NetworkConfig) -> Result<reqwest::Client, CoreError> {
    let mut builder = reqwest::Client::builder()
        .connect_timeout(config.connect_timeout)
        .read_timeout(config.timeout)
        .redirect(redirect::Policy::limited(10))
        .user_agent(&config.user_agent);

    if let Some(proxy_url) = &config.proxy {
        let proxy =
            reqwest::Proxy::all(proxy_url.as_str()).map_err(|e| CoreError::DownloadFailed {
                url: proxy_url.clone(),
                status: 0,
                reason: format!("invalid proxy URL: {e}"),
            })?;
        builder = builder.proxy(proxy);
    }

    builder.build().map_err(|e| CoreError::DownloadFailed {
        url: String::new(),
        status: 0,
        reason: format!("failed to build HTTP client: {e}"),
    })
}
