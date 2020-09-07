use super::definitions::Filter;
use super::definitions::Granularity;
use super::DataSource;
use super::group_by::PostAggregation;
use crate::query::definitions::Aggregation;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "queryType", rename = "timeseries")]
pub struct Timeseries {
    pub data_source: DataSource,
    pub granularity: Granularity,
    pub descending: bool,
    pub intervals: Vec<String>,
    pub filter: Option<Filter>,
    pub aggregations: Vec<Aggregation>,
    pub post_aggregations: Vec<PostAggregation>,
    pub limit: Option<usize>,
    pub context: std::collections::HashMap<String, String>,
}
