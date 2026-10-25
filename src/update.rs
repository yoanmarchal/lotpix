// Update check: compares the application version with the latest release published on
// GitHub. The application downloads and installs nothing by itself.
use std::fmt;
use std::time::Duration;

/// Page of the latest release, opened by the "Download" button. The address is fixed:
/// nothing returned by the network decides what the browser opens.
pub const DOWNLOAD_URL: &str = "https://github.com/yoanmarchal/lotpix/releases/latest";

// Without an answer within this delay, the check is silently abandoned
const TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

impl Version {
    /// Reads "2.1.0" or "v2.1.0". Any other format (pre-release, free text) is refused.
    pub fn parse(text: &str) -> Option<Self> {
        let mut parts = text.trim().trim_start_matches('v').split('.');
        let mut next = || parts.next()?.parse().ok();
        let version = Self {
            major: next()?,
            minor: next()?,
            patch: next()?,
        };
        parts.next().is_none().then_some(version)
    }

    pub fn current() -> Self {
        Self::parse(env!("CARGO_PKG_VERSION")).expect("Cargo.toml version in x.y.z format")
    }
}

impl fmt::Display for Version {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Version given by the address of a release, for example `…/releases/tag/v2.1.0`.
fn version_from_release_url(url: &str) -> Option<Version> {
    let (_, tag) = url.rsplit_once("/releases/tag/")?;
    Version::parse(tag)
}

/// Address to which GitHub redirects the "latest release" page.
///
/// The redirect of the web page is read rather than calling the API: the API is limited to
/// 60 requests per hour and per IP address, quickly reached behind a company network.
fn fetch_latest_release_url() -> Option<String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(TIMEOUT))
        // The expected answer is the redirect itself: do not follow it
        .max_redirects(0)
        .max_redirects_will_error(false)
        .build()
        .into();
    let response = agent
        .head(DOWNLOAD_URL)
        .header("User-Agent", concat!("lotpix/", env!("CARGO_PKG_VERSION")))
        .call()
        .ok()?;
    Some(
        response
            .headers()
            .get("location")?
            .to_str()
            .ok()?
            .to_owned(),
    )
}

/// Newer version than the application's, if there is one. Blocking call, to run outside
/// the interface thread. Offline or on error, there is simply nothing to
/// offer.
pub fn check() -> Option<Version> {
    let latest = version_from_release_url(&fetch_latest_release_url()?)?;
    (latest > Version::current()).then_some(latest)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn version(major: u64, minor: u64, patch: u64) -> Version {
        Version {
            major,
            minor,
            patch,
        }
    }

    #[test]
    fn parses_versions_with_or_without_prefix() {
        assert_eq!(Version::parse("2.1.0"), Some(version(2, 1, 0)));
        assert_eq!(Version::parse("v10.20.30"), Some(version(10, 20, 30)));
        assert_eq!(Version::parse(" v2.0.1\n"), Some(version(2, 0, 1)));
    }

    #[test]
    fn rejects_anything_that_is_not_three_numbers() {
        for text in ["", "v2", "2.1", "2.1.0.4", "2.1.0-beta", "latest", "2.x.0"] {
            assert_eq!(Version::parse(text), None, "{text}");
        }
    }

    #[test]
    fn orders_versions_numerically() {
        assert!(version(2, 10, 0) > version(2, 9, 9));
        assert!(version(3, 0, 0) > version(2, 99, 99));
        assert!(version(2, 0, 1) > version(2, 0, 0));
    }

    #[test]
    fn reads_the_current_version() {
        assert_eq!(Version::current().to_string(), env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn reads_the_version_from_a_release_url() {
        let repository = "https://github.com/yoanmarchal/lotpix";

        assert_eq!(
            version_from_release_url(&format!("{repository}/releases/tag/v2.1.0")),
            Some(version(2, 1, 0))
        );
        assert_eq!(
            version_from_release_url("/yoanmarchal/lotpix/releases/tag/v3.0.12"),
            Some(version(3, 0, 12))
        );
    }

    #[test]
    fn ignores_urls_that_do_not_name_a_release_version() {
        let repository = "https://github.com/yoanmarchal/lotpix";

        for url in [
            String::new(),
            // Repository without any release: GitHub redirects to the empty list
            format!("{repository}/releases"),
            format!("{repository}/releases/tag/nightly"),
            format!("{repository}/releases/tag/v3.0.0-rc.1"),
            format!("{repository}/releases/tag/v2.1.0/extra"),
            "https://github.com/login".to_owned(),
        ] {
            assert_eq!(version_from_release_url(&url), None, "{url}");
        }
    }

    // Really calls GitHub: cargo test -- --ignored queries_github
    #[test]
    #[ignore = "network access"]
    fn queries_github() {
        let url = fetch_latest_release_url().expect("GitHub redirect");

        assert!(version_from_release_url(&url).is_some(), "{url}");
    }
}
