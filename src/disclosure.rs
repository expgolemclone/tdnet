use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Disclosure {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub time: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub code: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub title: String,
    pub pdf: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub xbrl: Option<String>,
}
