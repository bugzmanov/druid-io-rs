use crate::query::DataSource;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "queryType", rename = "segmentMetadata")]
#[serde(rename_all = "camelCase")]
pub struct SegmentMetadata {
    pub data_source: DataSource,
    pub intervals: Vec<String>,
    pub to_include: ToInclude,
    pub merge: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub analysis_types: Vec<AnalysisType>,
    pub lenient_aggregator_merge: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum ToInclude {
    All,
    None,
    List { columns: Vec<String> },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum AnalysisType {
    Cardinality,
    Minmax,
    Size,
    Interval,
    TimestampSpec,
    QueryGranularity,
    Aggregators,
    Rollup,
}
