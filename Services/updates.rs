// Copyright (C) 2026 slatkisasa
// SPDX-License-Identifier: AGPL-3.0-only
use anyhow::{Context, Result, ensure};
use semver::Version;
use serde::Deserialize;
use std::time::Duration;

pub const RELEASE_URL: &str = "https://github.com/slatkisasa/sessiondeck/releases/latest";
const API_URL: &str = "https://api.github.com/repos/slatkisasa/sessiondeck/releases/latest";
pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    draft: bool,
    prerelease: bool,
}

#[derive(Debug, PartialEq)]
pub enum CheckResult {
    Available(Version),
    Current,
    NoRelease,
}

fn compare_release(release: Release, current: &str) -> Result<CheckResult> {
    ensure!(
        !release.draft && !release.prerelease,
        "Expected a published stable release"
    );
    let latest = Version::parse(
        release
            .tag_name
            .strip_prefix('v')
            .unwrap_or(&release.tag_name),
    )
    .context("Invalid release version")?;
    ensure!(latest.pre.is_empty(), "Expected a stable release version");
    let installed = Version::parse(current)?;
    Ok(if latest.cmp_precedence(&installed).is_gt() {
        CheckResult::Available(latest)
    } else {
        CheckResult::Current
    })
}

/// Public release metadata only: no credentials, account data, or local paths.
pub fn check() -> Result<CheckResult> {
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(10))
        .redirects(0)
        .build();
    let response = match agent
        .get(API_URL)
        .set(
            "User-Agent",
            concat!("SessionDeck/", env!("CARGO_PKG_VERSION")),
        )
        .set("Accept", "application/vnd.github+json")
        .call()
    {
        Ok(response) => response,
        Err(ureq::Error::Status(404, _)) => return Ok(CheckResult::NoRelease),
        Err(error) => return Err(error.into()),
    };
    // Bound memory usage even if the server returns an unexpected response.
    let release: Release =
        serde_json::from_reader(std::io::Read::take(response.into_reader(), 1024 * 1024))
            .context("Could not read release metadata")?;
    compare_release(release, CURRENT_VERSION)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release(tag: &str) -> Release {
        Release {
            tag_name: tag.into(),
            draft: false,
            prerelease: false,
        }
    }

    #[test]
    fn compares_versions_numerically_and_never_offers_a_downgrade() {
        assert_eq!(
            compare_release(release("v0.10.0"), "0.3.0").unwrap(),
            CheckResult::Available(Version::parse("0.10.0").unwrap())
        );
        for tag in ["v0.3.0", "0.2.0", "0.3.0+build"] {
            assert_eq!(
                compare_release(release(tag), "0.3.0").unwrap(),
                CheckResult::Current
            );
        }
    }

    #[test]
    fn rejects_invalid_and_unpublished_versions() {
        for tag in ["latest", "v0.4.0-beta.1", ""] {
            assert!(compare_release(release(tag), "0.3.0").is_err());
        }
        for (draft, prerelease) in [(true, false), (false, true)] {
            assert!(
                compare_release(
                    Release {
                        draft,
                        prerelease,
                        ..release("v0.4.0")
                    },
                    "0.3.0"
                )
                .is_err()
            );
        }
    }
}
