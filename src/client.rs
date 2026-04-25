use std::time::Duration;

use reqwest::Client;

/// Build a tuned reqwest Client for translation workloads.
///
/// Configuration:
/// - Pool max idle per host: 50 connections
/// - Pool idle timeout: 120 seconds
/// - TCP keepalive: 30 seconds (pins reqwest default, cross-platform)
/// - Request timeout: 30 seconds
/// - HTTP/2 disabled by default (not opted into)
///
/// Cloning is cheap - share one instance per CLI invocation.
#[must_use]
pub fn build_client() -> Client {
    Client::builder()
        .pool_max_idle_per_host(50)
        .pool_idle_timeout(Duration::from_secs(120))
        .tcp_keepalive(Duration::from_secs(30))
        .timeout(Duration::from_secs(30))
        .build()
        .expect("reqwest client builder should always succeed with valid settings")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_builds() {
        let client = build_client();
        let _ = client.clone();
    }

    #[test]
    fn client_clones_are_cheap() {
        let client = build_client();
        let _clone1 = client.clone();
        let _clone2 = client.clone();
        let _clone3 = client.clone();
    }
}
