// Vérification des mises à jour : compare la version de l'application à la dernière release
// publiée sur GitHub. L'application ne télécharge ni n'installe rien elle-même.
use std::fmt;
use std::time::Duration;

/// Page de la dernière release, ouverte par le bouton « Télécharger ». L'adresse est fixe :
/// rien de ce que renvoie le réseau ne décide de ce que le navigateur ouvre.
pub const DOWNLOAD_URL: &str = "https://github.com/yoanmarchal/lotpix/releases/latest";

// Sans réponse dans ce délai, la vérification est abandonnée en silence
const TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

impl Version {
    /// Lit « 2.1.0 » ou « v2.1.0 ». Tout autre format (préversion, texte libre) est refusé.
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
        Self::parse(env!("CARGO_PKG_VERSION")).expect("version de Cargo.toml au format x.y.z")
    }
}

impl fmt::Display for Version {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Version désignée par l'adresse d'une release, par exemple `…/releases/tag/v2.1.0`.
fn version_from_release_url(url: &str) -> Option<Version> {
    let (_, tag) = url.rsplit_once("/releases/tag/")?;
    Version::parse(tag)
}

/// Adresse vers laquelle GitHub redirige la page « dernière release ».
///
/// On lit la redirection de la page web plutôt que d'appeler l'API : celle-ci est limitée à
/// 60 requêtes par heure et par adresse IP, vite atteintes derrière un réseau d'entreprise.
fn fetch_latest_release_url() -> Option<String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(TIMEOUT))
        // La réponse attendue est la redirection elle-même : ne pas la suivre
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

/// Version plus récente que celle de l'application, s'il y en a une. Appel bloquant, à lancer
/// hors du fil de l'interface. Hors ligne ou en cas d'erreur, il n'y a simplement rien à
/// proposer.
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
            // Dépôt sans aucune release : GitHub renvoie vers la liste, vide
            format!("{repository}/releases"),
            format!("{repository}/releases/tag/nightly"),
            format!("{repository}/releases/tag/v3.0.0-rc.1"),
            format!("{repository}/releases/tag/v2.1.0/extra"),
            "https://github.com/login".to_owned(),
        ] {
            assert_eq!(version_from_release_url(&url), None, "{url}");
        }
    }

    // Appelle réellement GitHub : cargo test -- --ignored queries_github
    #[test]
    #[ignore = "accès réseau"]
    fn queries_github() {
        let url = fetch_latest_release_url().expect("redirection de GitHub");

        assert!(version_from_release_url(&url).is_some(), "{url}");
    }
}
