use std::sync::LazyLock;

use semver::Version;

pub mod api;
pub mod config;
pub mod require;
pub mod stages;

pub static HOST_API_VERSION: LazyLock<Version> = LazyLock::new(|| {
    Version::parse(env!("CARGO_PKG_VERSION")).expect("CARGO_PKG_VERSION is not valid semver")
});
