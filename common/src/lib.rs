#[cfg(feature = "backend")]
pub mod backends;
#[cfg(feature = "claims")]
pub mod claims;
pub mod constants;
pub mod database;
#[cfg(feature = "service")]
pub mod deployment;
#[cfg(feature = "service")]
use uuid::Uuid;
#[cfg(feature = "service")]
pub type DeploymentId = Uuid;
#[cfg(feature = "service")]
pub mod log;
#[cfg(feature = "service")]
pub use log::LogItem;
#[cfg(feature = "models")]
pub mod models;
pub mod resource;
pub mod secrets;
pub use secrets::{Secret, SecretStore};
#[cfg(feature = "tracing")]
pub mod tracing;
#[cfg(feature = "wasm")]
pub mod wasm;

use std::fmt::Debug;

use anyhow::bail;
use serde::{Deserialize, Serialize};
#[cfg(feature = "openapi")]
use utoipa::openapi::{Object, ObjectBuilder};
use zeroize::Zeroize;

pub type ApiUrl = String;
pub type Host = String;

#[derive(Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "persist", derive(PartialEq, Eq, Hash, sqlx::Type))]
#[cfg_attr(feature = "persist", serde(transparent))]
#[cfg_attr(feature = "persist", sqlx(transparent))]
pub struct ApiKey(String);

impl Zeroize for ApiKey {
    fn zeroize(&mut self) {
        self.0.zeroize()
    }
}

impl ApiKey {
    pub fn parse(key: &str) -> anyhow::Result<Self> {
        let key = key.trim();

        let mut errors = vec![];
        if !key.chars().all(char::is_alphanumeric) {
            errors.push("The API key should consist of only alphanumeric characters.");
        };

        if key.len() != 16 {
            errors.push("The API key should be exactly 16 characters in length.");
        };

        if !errors.is_empty() {
            let message = errors.join("\n");
            bail!("Invalid API key:\n{message}")
        }

        Ok(Self(key.to_string()))
    }

    #[cfg(feature = "persist")]
    pub fn generate() -> Self {
        use rand::distributions::{Alphanumeric, DistString};

        Self(Alphanumeric.sample_string(&mut rand::thread_rng(), 16))
    }
}

impl AsRef<str> for ApiKey {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Holds the input for a DB resource
#[derive(Deserialize, Serialize, Default)]
pub struct DbInput {
    pub local_uri: Option<String>,
}

/// Holds the output for a DB resource
#[derive(Deserialize, Serialize)]
pub enum DbOutput {
    Info(DatabaseReadyInfo),
    Local(String),
}

/// Holds the details for a database connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseReadyInfo {
    engine: String,
    role_name: String,
    role_password: Secret<String>,
    database_name: String,
    port: String,
    address_private: String,
    address_public: String,
}

impl DatabaseReadyInfo {
    pub fn new(
        engine: String,
        role_name: String,
        role_password: String,
        database_name: String,
        port: String,
        address_private: String,
        address_public: String,
    ) -> Self {
        Self {
            engine,
            role_name,
            role_password: Secret::new(role_password),
            database_name,
            port,
            address_private,
            address_public,
        }
    }
    /// For connecting to the db from inside the Shuttle network
    pub fn connection_string_private(&self) -> String {
        format!(
            "{}://{}:{}@{}:{}/{}",
            self.engine,
            self.role_name,
            self.role_password.expose(),
            self.address_private,
            self.port,
            self.database_name,
        )
    }
    /// For connecting to the db from the Internet
    pub fn connection_string_public(&self, show_password: bool) -> String {
        format!(
            "{}://{}:{}@{}:{}/{}",
            self.engine,
            self.role_name,
            if show_password {
                self.role_password.expose()
            } else {
                self.role_password.redacted()
            },
            self.address_public,
            self.port,
            self.database_name,
        )
    }
}

#[cfg(feature = "openapi")]
pub fn ulid_type() -> Object {
    ObjectBuilder::new()
        .schema_type(utoipa::openapi::SchemaType::String)
        .format(Some(utoipa::openapi::SchemaFormat::Custom(
            "ulid".to_string(),
        )))
        .description(Some("String represention of an Ulid according to the spec found here: https://github.com/ulid/spec."))
        .build()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionInfo {
    /// Version of gateway
    pub gateway: semver::Version,
    /// Latest version of cargo-shuttle compatible with this gateway.
    pub cargo_shuttle: semver::Version,
    /// Latest version of shuttle-deployer compatible with this gateway.
    pub deployer: semver::Version,
    /// Latest version of shuttle-runtime compatible with the above deployer.
    pub runtime: semver::Version,
}

/// Check if two versions are compatible based on the rule used by cargo:
/// "Versions `a` and `b` are compatible if their left-most nonzero digit is the same."
pub fn semvers_are_compatible(a: &semver::Version, b: &semver::Version) -> bool {
    if a.major != 0 || b.major != 0 {
        a.major == b.major
    } else if a.minor != 0 || b.minor != 0 {
        a.minor == b.minor
    } else {
        a.patch == b.patch
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::str::FromStr;

    proptest! {
        #[test]
        // The API key should be a 16 character alphanumeric string.
        fn parses_valid_api_keys(s in "[a-zA-Z0-9]{16}") {
            ApiKey::parse(&s).unwrap();
        }
    }

    #[cfg(feature = "persist")]
    #[test]
    fn generated_api_key_is_valid() {
        let key = ApiKey::generate();

        assert!(ApiKey::parse(key.as_ref()).is_ok());
    }

    #[test]
    #[should_panic(expected = "The API key should be exactly 16 characters in length.")]
    fn invalid_api_key_length() {
        ApiKey::parse("tooshort").unwrap();
    }

    #[test]
    #[should_panic(expected = "The API key should consist of only alphanumeric characters.")]
    fn non_alphanumeric_api_key() {
        ApiKey::parse("dh9z58jttoes3qv@").unwrap();
    }

    #[test]
    fn semver_compatibility_check_works() {
        let semver_tests = &[
            ("1.0.0", "1.0.0", true),
            ("1.8.0", "1.0.0", true),
            ("0.1.0", "0.2.1", false),
            ("0.9.0", "0.2.0", false),
        ];
        for (version_a, version_b, are_compatible) in semver_tests {
            let version_a = semver::Version::from_str(version_a).unwrap();
            let version_b = semver::Version::from_str(version_b).unwrap();
            assert_eq!(
                super::semvers_are_compatible(&version_a, &version_b),
                *are_compatible
            );
        }
    }
}
