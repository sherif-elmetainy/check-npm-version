use chrono::{DateTime, Utc};
use semver::Version;

#[derive(Debug)]
pub struct PackageVersionInfo {
    pub name: String,
    pub declared: String,
    pub latest: Option<Version>,
    pub latest_time: Option<DateTime<Utc>>,
    pub latest_same_major: Option<Version>,
    pub latest_same_major_time: Option<DateTime<Utc>>,
    pub latest_same_minor: Option<Version>,
    pub latest_same_minor_time: Option<DateTime<Utc>>,
    pub latest_satisfying: Option<Version>,
    pub latest_satisfying_time: Option<DateTime<Utc>>,
}
