use pubgrub::range::Range;

use pep440_rs::{Operator, PreRelease, Version, VersionSpecifier};

use crate::ResolveError;

/// A range of versions that can be used to satisfy a requirement.
#[derive(Debug)]
pub(crate) struct PubGrubSpecifier(Range<Version>);

impl From<PubGrubSpecifier> for Range<Version> {
    /// Convert a PubGrub specifier to a range of versions.
    fn from(specifier: PubGrubSpecifier) -> Self {
        specifier.0
    }
}

impl TryFrom<&VersionSpecifier> for PubGrubSpecifier {
    type Error = ResolveError;

    /// Convert a PEP 508 specifier to a PubGrub-compatible version range.
    fn try_from(specifier: &VersionSpecifier) -> Result<Self, ResolveError> {
        let ranges = match specifier.operator() {
            Operator::Equal => {
                let version = specifier.version().clone();
                Range::singleton(version)
            }
            Operator::ExactEqual => {
                let version = specifier.version().clone();
                Range::singleton(version)
            }
            Operator::NotEqual => {
                let version = specifier.version().clone();
                Range::singleton(version).complement()
            }
            Operator::TildeEqual => {
                let [rest @ .., last, _] = specifier.version().release() else {
                    return Err(ResolveError::InvalidTildeEquals(specifier.clone()));
                };
                let upper = Version::new(rest.iter().chain([&(last + 1)]))
                    .with_epoch(specifier.version().epoch())
                    .with_dev(Some(0));
                let version = specifier.version().clone();
                Range::from_range_bounds(version..upper)
            }
            Operator::LessThan => {
                let version = specifier.version().clone();
                if version.any_prerelease() {
                    Range::strictly_lower_than(version)
                } else {
                    // Per PEP 440: "The exclusive ordered comparison <V MUST NOT allow a
                    // pre-release of the specified version unless the specified version is itself a
                    // pre-release.
                    Range::strictly_lower_than(version.with_min(Some(0)))
                }
            }
            Operator::LessThanEqual => {
                let version = specifier.version().clone();
                Range::lower_than(version)
            }
            Operator::GreaterThan => {
                // Per PEP 440: "The exclusive ordered comparison >V MUST NOT allow a post-release of
                // the given version unless V itself is a post release."
                let version = specifier.version().clone();
                if let Some(dev) = version.dev() {
                    Range::higher_than(version.with_dev(Some(dev + 1)))
                } else if let Some(post) = version.post() {
                    Range::higher_than(version.with_post(Some(post + 1)))
                } else {
                    Range::strictly_higher_than(version.with_max(Some(0)))
                }
            }
            Operator::GreaterThanEqual => {
                let version = specifier.version().clone();
                Range::higher_than(version)
            }
            Operator::EqualStar => {
                let low = specifier.version().clone().with_dev(Some(0));
                let mut high = low.clone();
                if let Some(post) = high.post() {
                    high = high.with_post(Some(post + 1));
                } else if let Some(pre) = high.pre() {
                    high = high.with_pre(Some(PreRelease {
                        kind: pre.kind,
                        number: pre.number + 1,
                    }));
                } else {
                    let mut release = high.release().to_vec();
                    *release.last_mut().unwrap() += 1;
                    high = high.with_release(release);
                }
                Range::from_range_bounds(low..high)
            }
            Operator::NotEqualStar => {
                let low = specifier.version().clone().with_dev(Some(0));
                let mut high = low.clone();
                if let Some(post) = high.post() {
                    high = high.with_post(Some(post + 1));
                } else if let Some(pre) = high.pre() {
                    high = high.with_pre(Some(PreRelease {
                        kind: pre.kind,
                        number: pre.number + 1,
                    }));
                } else {
                    let mut release = high.release().to_vec();
                    *release.last_mut().unwrap() += 1;
                    high = high.with_release(release);
                }
                Range::from_range_bounds(low..high).complement()
            }
        };

        Ok(Self(ranges))
    }
}
