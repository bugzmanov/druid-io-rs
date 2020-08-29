use anyhow::{Context, Result};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DruidClientError {
    #[error("http connection error")]
    HttpConnection { source: reqwest::Error },
    #[error("the data for key `{0}` is not available")]
    Redaction(String),
    #[error("invalid header (expected {expected:?}, found {found:?})")]
    InvalidHeader { expected: String, found: String },
    #[error("couldn't parse object to/from json")]
    ParsingError { source: serde_json::Error },
    #[error("unknown data store error")]
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OutputType {
    STRING,
    LONG,
    FLOAT,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum Dimension {
    Default {
        dimension: String,
        outputName: String,
        outputType: OutputType,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Granularity {
    ALL,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum Aggregation {
    Count { name: String },
    LongSum { name: String, field_name: String },
    DoubleSum { name: String, field_name: String },
    FloatSum { name: String, field_name: String },
    LongMax { name: String, field_name: String },
    DoubleMax { name: String, field_name: String },
    FloatMax { name: String, field_name: String },
    LongMin { name: String, field_name: String },
    FloatMin { name: String, field_name: String },
    DoubleMin { name: String, field_name: String },
    LongFirst { name: String, field_name: String },
    FloatFirst { name: String, field_name: String },
    DoubleFirst { name: String, field_name: String },
    LongLast { name: String, field_name: String },
    FloatLast { name: String, field_name: String },
    DoubleLast { name: String, field_name: String },
    // StringFirst{name: String, field_name: String, max_string_bytes: Option<usize>, filter_null_values: bool},
    // StringLastPname: String, field_name: String, max_string_bytes: Option<usize>, filter_null_values: bool},
    // ThetaSketch(String, String),
    // HyperUnique(String, String),
    // Cardinality(String, String),
    // Filtered(String, String),
    // Javascript(String, String),
}

// pub enum Interval {

// }
#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum DataSource {
    Table { name: String },
}
