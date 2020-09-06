use crate::query::DataSource;
use crate::query::Filter;
use crate::query::Ordering;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "queryType", rename = "scan")]
#[serde(rename_all = "camelCase")]
pub struct Scan {
    pub data_source: DataSource,
    pub intervals: Vec<String>,
    pub result_format: ResultFormat,
    pub filter: Option<Filter>,
    pub columns: Vec<String>,
    pub batch_size: usize,
    pub limit: Option<usize>,
    pub ordering: Option<Ordering>,
    pub context: std::collections::HashMap<String, String>,
}


#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ResultFormat {
    List,
    CompactedList,
    ValueVector,
}