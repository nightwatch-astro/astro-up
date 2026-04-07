use reqwest::redirect;

use crate::config::NetworkConfig;
use crate::error::CoreError;

/// Default user-agent string including the crate version.
pub fn default_user_agent() -> String {
    format!("{}/{}", crate::CRATE_NAME, crate::version())
}

/// Build a configured reqwest client from network settings.
pub fn build_client(config: &NetworkConfig) -> Result<reqwest::Client, CoreError> {
    tracing::debug!(
        connect_timeout = ?config.connect_timeout,
        read_timeout = ?config.timeout,
        has_proxy = config.proxy.is_some(),
        "building HTTP client"
    );
    let ua = if config.user_agent.is_empty() {
        default_user_agent()
    } else {
        config.user_agent.clone()
    };

    let mut builder = reqwest::Client::builder()
        .connect_timeout(config.connect_timeout)
        .read_timeout(config.timeout)
        .redirect(redirect::Policy::limited(10))
        .user_agent(&ua);

    if let Some(proxy_url) = &config.proxy {
        let proxy =
            reqwest::Proxy::all(proxy_url.as_str()).map_err(|e| CoreError::DownloadFailed {
                url: proxy_url.clone(),
                status: 0,
                reason: format!("invalid proxy URL: {e}"),
            })?;
        builder = builder.proxy(proxy);
    }

    tracing::debug!(user_agent = %ua, "HTTP client configured");
    builder.build().map_err(|e| CoreError::DownloadFailed {
        url: String::new(),
        status: 0,
        reason: format!("failed to build HTTP client: {e}"),
    })
}
