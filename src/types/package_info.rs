use std::collections::BTreeMap;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PackageInfo {
    pub time: BTreeMap<String, String>,
}
