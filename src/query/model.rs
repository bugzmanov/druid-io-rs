use crate::query::DataSource;
use crate::query::Filter;
use serde::{Deserialize, Serialize};
use super::{scan::Scan, group_by::GroupBy, search::Search, time_boundary::TimeBoundary, segment_metadata::SegmentMetadata, top_n::TopN};

// }
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
#[serde(rename_all = "camelCase")]
pub enum Query {
    #[serde(rename_all = "camelCase")]
    TopN(TopN),
    GroupBy(GroupBy),
    Scan(Scan),
    Search(Search),
    TimeBoundary(TimeBoundary),
    SegmentMetadata(SegmentMetadata),
}
impl From<TopN> for Query {
    fn from(query: TopN) -> Self {
       Query::TopN(query) 
    }
}
impl From<GroupBy> for Query {
    fn from(query: GroupBy) -> Self {
       Query::GroupBy(query) 
    }
}
impl From<Scan> for Query {
    fn from(scan: Scan) -> Self {
       Query::Scan(scan) 
    }
}
impl From<Search> for Query {
    fn from(query: Search) -> Self {
       Query::Search(query) 
    }
}
impl From<TimeBoundary> for Query {
    fn from(query: TimeBoundary) -> Self {
       Query::TimeBoundary(query) 
    }
}
impl From<SegmentMetadata> for Query {
    fn from(query: SegmentMetadata) -> Self {
       Query::SegmentMetadata(query) 
    }
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "queryType", rename = "dataSourceMetadata")]
pub struct DataSourceMetadata {
    pub data_source: DataSource,
    pub context: std::collections::HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum HllType {
    #[allow(non_camel_case_types)]
    HLL_4,
    #[allow(non_camel_case_types)]
    HLL_6,
    #[allow(non_camel_case_types)]
    HLL_8,
}
#[rustfmt::skip]
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum Aggregation {
    Count { name: String },
    #[serde(rename_all = "camelCase")]
    LongSum { name: String, field_name: String },
    #[serde(rename_all = "camelCase")]
    DoubleSum { name: String, field_name: String },
    #[serde(rename_all = "camelCase")]
    FloatSum { name: String, field_name: String },
    #[serde(rename_all = "camelCase")]
    LongMax { name: String, field_name: String },
    #[serde(rename_all = "camelCase")]
    DoubleMax { name: String, field_name: String },
    #[serde(rename_all = "camelCase")]
    FloatMax { name: String, field_name: String },
    #[serde(rename_all = "camelCase")]
    LongMin { name: String, field_name: String },
    #[serde(rename_all = "camelCase")]
    FloatMin { name: String, field_name: String },
    #[serde(rename_all = "camelCase")]
    DoubleMin { name: String, field_name: String },
    #[serde(rename_all = "camelCase")]
    LongFirst { name: String, field_name: String },
    #[serde(rename_all = "camelCase")]
    FloatFirst { name: String, field_name: String },
    #[serde(rename_all = "camelCase")]
    DoubleFirst { name: String, field_name: String },
    #[serde(rename_all = "camelCase")]
    LongLast { name: String, field_name: String },
    #[serde(rename_all = "camelCase")]
    FloatLast { name: String, field_name: String },
    #[serde(rename_all = "camelCase")]
    DoubleLast { name: String, field_name: String },
    #[serde(rename_all = "camelCase")]
    StringFirst { name: String, field_name: String, max_string_bytes: usize },
    #[serde(rename_all = "camelCase")]
    StringLast { name: String, field_name: String, max_string_bytes: usize },

    #[serde(rename_all = "camelCase")]
    DoubleAny { name: String, field_name: String },
    #[serde(rename_all = "camelCase")]
    FloatAny { name: String, field_name: String },
    #[serde(rename_all = "camelCase")]
    LongAny { name: String, field_name: String },
    #[serde(rename_all = "camelCase")]
    StringAny { name: String, field_name: String },

    #[serde(rename_all = "camelCase")]
    Javascript { name: String, field_names: Vec<String>, fn_aggregate: String, fn_combine: String, fn_reset: String},
    
    #[serde(rename_all = "camelCase")]
    ThetaSketch {name: String, field_name: String, is_input_theta_sketch: bool, size: usize},


    #[serde(rename_all = "camelCase")]
    HLLSketchBuild { name: String, field_name: String, lg_k: usize, lgt_hll_type: HllType, round: bool},

    #[serde(rename_all = "camelCase")]
    Cardinality { name: String, fields: Vec<String>, by_row: bool, round: bool},

    #[serde(rename_all = "camelCase")]
    HyperUnique { name: String, field_name: String, is_input_hyper_unique: bool, round: bool},

    Filtered { filter: Filter, aggregator: Box<Aggregation>}
}

// todo: macro
impl Aggregation {
    pub fn count(name: &str) -> Aggregation {
        Aggregation::Count {
            name: name.to_string(),
        }
    }
    pub fn long_sum(name: &str, field_name: &str) -> Aggregation {
        Aggregation::LongSum {
            name: name.to_string(),
            field_name: field_name.to_string(),
        }
    }
    pub fn double_sum(name: &str, field_name: &str) -> Aggregation {
        Aggregation::DoubleSum {
            name: name.to_string(),
            field_name: field_name.to_string(),
        }
    }
    pub fn float_sum(name: &str, field_name: &str) -> Aggregation {
        Aggregation::FloatSum {
            name: name.to_string(),
            field_name: field_name.to_string(),
        }
    }
    pub fn long_max(name: &str, field_name: &str) -> Aggregation {
        Aggregation::LongMax {
            name: name.to_string(),
            field_name: field_name.to_string(),
        }
    }
    pub fn double_max(name: &str, field_name: &&str) -> Aggregation {
        Aggregation::DoubleMax {
            name: name.to_string(),
            field_name: field_name.to_string(),
        }
    }
    pub fn float_max(name: &str, field_name: &str) -> Aggregation {
        Aggregation::FloatMax {
            name: name.to_string(),
            field_name: field_name.to_string(),
        }
    }
    pub fn long_min(name: &str, field_name: &str) -> Aggregation {
        Aggregation::LongMin {
            name: name.to_string(),
            field_name: field_name.to_string(),
        }
    }
    pub fn float_min(name: &str, field_name: &str) -> Aggregation {
        Aggregation::FloatMin {
            name: name.to_string(),
            field_name: field_name.to_string(),
        }
    }
    pub fn double_min(name: &str, field_name: &str) -> Aggregation {
        Aggregation::DoubleMin {
            name: name.to_string(),
            field_name: field_name.to_string(),
        }
    }
    pub fn long_first(name: &str, field_name: &str) -> Aggregation {
        Aggregation::LongFirst {
            name: name.to_string(),
            field_name: field_name.to_string(),
        }
    }
    pub fn float_first(name: &str, field_name: &str) -> Aggregation {
        Aggregation::FloatFirst {
            name: name.to_string(),
            field_name: field_name.to_string(),
        }
    }
    // pub fn double_first(name: &str, field_name: &str) -> Aggregation {}
    // pub fn long_last(name: &str, field_name: &str) -> Aggregation {}
    // pub fn float_last(name: &str, field_name: &str) -> Aggregation {}
    // pub fn double_last(name: &str, field_name: &str) -> Aggregation {}
    // pub fn string_first(name: &str, field_name: &str, max_string_bytes: usize) -> Aggregation {}
    // pub fn string_last(name: &str, field_name: &str, max_string_bytes: usize) -> Aggregation {}
    // pub fn double_any(name: &str, field_name: &str) -> Aggregation {}
    // pub fn float_any(name: &str, field_name: &str) -> Aggregation {}
    // pub fn long_any(name: &str, field_name: &str) -> Aggregation {}
    // pub fn string_any(name: &str, field_name: &str) -> Aggregation {}
}






#[serde(untagged)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum JsonNumber {
    Float(f32),
    Integer(isize)
}

impl From<f32> for JsonNumber {
    fn from(float: f32) -> Self {
        JsonNumber::Float(float)
    }
}

impl From<isize> for JsonNumber {
    fn from(integer: isize) -> Self {
        JsonNumber::Integer(integer)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum JsonAny {
    Float(f32),
    Integer(isize),
    STRING(String),
    Boolean(bool)
}

impl From<f32> for JsonAny {
    fn from(float: f32) -> Self {
        JsonAny::Float(float)
    }
}

impl From<isize> for JsonAny {
    fn from(integer: isize) -> Self {
        JsonAny::Integer(integer)
    }
}

impl From<bool> for JsonAny {
    fn from(boolean: bool) -> Self {
        JsonAny::Boolean(boolean)
    }
}

impl From<String> for JsonAny {
    fn from(str: String) -> Self {
        JsonAny::STRING(str)
    }
}

impl From<&str> for JsonAny {
    fn from(str: &str) -> Self {
        JsonAny::STRING(str.to_string())
    }
}