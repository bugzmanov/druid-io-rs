use crate::query::search::Search;
use group_by::GroupBy;
use scan::Scan;
use segment_metadata::SegmentMetadata;
use serde::{Deserialize, Serialize};
use time_boundary::TimeBoundary;
use top_n::TopN;

pub mod definitions;
pub mod group_by;
pub mod response;
pub mod scan;
pub mod search;
pub mod segment_metadata;
pub mod time_boundary;
pub mod top_n;

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
#[serde(rename_all = "camelCase")]
pub enum Query {
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

#[rustfmt::skip]
#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum DataSource {
    Table { name: String },
    Lookup { lookup: String },
    #[serde(rename_all = "camelCase")]
    Union { data_sources: Vec<String> },
    #[serde(rename_all = "camelCase")]
    Inline {
         column_names: Vec<String>,
         rows: Vec<Vec<String>>,
    },
    #[serde(rename_all = "camelCase")]
    Query { query: Box<Query> },
    #[serde(rename_all = "camelCase")]
    // left: table, join, lookup, query, or inline
    // right: lookup, query, or inline
    Join {left: Box<DataSource>, right: Box<DataSource>, right_prefix: String, condition: String, join_type: JoinType } 
}

pub struct JoinBuilder {
    left: Option<DataSource>,
    right: Option<DataSource>,
    right_prefix: Option<String>,
    condition: Option<String>,
    join_type: JoinType,
}

impl JoinBuilder {
    pub fn new(join_type: JoinType) -> Self {
        JoinBuilder {
            left: None,
            right: None,
            right_prefix: None,
            condition: None,
            join_type: join_type,
        }
    }
    pub fn left(mut self, left: DataSource) -> Self {
        self.left = Some(left);
        self
    }
    pub fn right(mut self, right: DataSource, right_prefix: &str) -> Self {
        self.right = Some(right);
        self.right_prefix = Some(right_prefix.to_string());
        self
    }
    pub fn condition(mut self, condition: &str) -> Self {
        self.condition = Some(condition.to_string());
        self
    }
    pub fn build(&mut self) -> Option<DataSource> {
        if let (Some(left), Some(right), Some(condition), Some(right_prefix)) = (
            self.left.take(),
            self.right.take(),
            self.condition.take(),
            self.right_prefix.take(),
        ) {
            Some(DataSource::Join {
                join_type: self.join_type.clone(),
                left: Box::new(left),
                right: Box::new(right),
                condition: condition,
                right_prefix: right_prefix,
            })
        } else {
            return None;
        }
    }
}

impl DataSource {
    pub fn table(name: &str) -> DataSource {
        DataSource::Table { name: name.into() }
    }
    pub fn lookup(name: &str) -> DataSource {
        DataSource::Lookup {
            lookup: name.into(),
        }
    }
    pub fn union(sources: Vec<&str>) -> DataSource {
        DataSource::Union {
            data_sources: sources.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn query(query: Query) -> DataSource {
        DataSource::Query {
            query: Box::new(query),
        }
    }

    pub fn join(join_type: JoinType) -> JoinBuilder {
        JoinBuilder::new(join_type)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum JoinType {
    Inner,
    Left,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "queryType", rename = "dataSourceMetadata")]
pub struct DataSourceMetadata {
    pub data_source: DataSource,
    pub context: std::collections::HashMap<String, String>,
}

#[serde(untagged)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum JsonNumber {
    Float(f32),
    Integer(isize),
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
    Boolean(bool),
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
