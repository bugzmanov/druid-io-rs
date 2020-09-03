use crate::query::DataSource;
use crate::query::Dimension;
use crate::query::Filter;
use crate::query::Granularity;
use crate::query::Ordering;
use serde::{Deserialize, Serialize};
use super::SortingOrder;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "queryType", rename = "search")]
pub struct Search {
    pub data_source: DataSource,
    pub granularity: Granularity,
    pub filter: Option<Filter>,
    pub limit: usize,
    pub intervals: Vec<String>,
    pub search_dimensions: Vec<String>,
    pub query: SearchQuerySpec,
    pub sort: Option<SortingOrder>,
    pub context: std::collections::HashMap<String, String>,
}


#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum SearchQuerySpec {
    #[serde(rename_all = "camelCase")]
    InsensitiveContains { value: String },
    #[serde(rename_all = "camelCase")]
    Fragment {
        case_sensitive: bool,
        values: Vec<String>,
    },
    #[serde(rename_all = "camelCase")]
    Contains { case_sensitive: bool, value: String },
    #[serde(rename_all = "camelCase")]
    Regex { pattern: String },
}

impl SearchQuerySpec {
    pub fn contains_insensitive(value: &str) -> Self {
        SearchQuerySpec::InsensitiveContains {
            value: value.to_string(),
        }
    }

    pub fn constain(value: &str, case_sensitive: bool) -> Self {
        SearchQuerySpec::Contains {
            value: value.to_string(),
            case_sensitive: case_sensitive,
        }
    }
    pub fn fragment(values: Vec<&str>, case_sensitive: bool) -> Self {
        SearchQuerySpec::Fragment {
            values: values.iter().map(|s| s.to_string()).collect(),
            case_sensitive: case_sensitive,
        }
    }

    pub fn regrex(pattern: &str) -> Self {
        SearchQuerySpec::Regex {
            pattern: pattern.to_string(),
        }
    }
}