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
    #[error("Server responded with an error")]
    ServerError { response: String },
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
#[serde(rename_all = "camelCase")]
pub enum Dimension {
    #[serde(rename_all = "camelCase")]
    Default {
        dimension: String,
        output_name: String,
        output_type: OutputType,
    },
    #[serde(rename_all = "camelCase")]
    Extraction {
        dimenstion: String,
        output_name: String,
        output_type: OutputType,
        extraction_fn: ExtractFN,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Granularity {
    All,
    None,
    Second,
    Minute,
    Fifteen_minute,
    Thirty_minute,
    Hour,
    Day,
    Week,
    Month,
    Quarter,
    Year,
    // #[serde(tag = "type")]
    Duration { duration: usize }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum HllType {
    HLL_4,
    HLL_6,
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
    StringFirst{name: String, field_name: String, max_string_bytes: usize },
    // StringLastPname: String, field_name: String, max_string_bytes: Option<usize>, filter_null_values: bool},
    #[serde(rename_all = "camelCase")]
    StringLast{name: String, field_name: String, max_string_bytes: usize },

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

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum Filter {
    #[serde(rename_all = "camelCase")]
    Selector {
        dimension: String,
        value: String,
        extract_fn: Option<ExtractFN>,
    },
}

#[rustfmt::skip]
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ExtractFN {
    #[serde(rename_all = "camelCase")]
    Regex { expr: String, index: usize, replace_missing_value: bool, replace_missing_value_with: Option<String>},
    #[serde(rename_all = "camelCase")]
    Partial { expr: String },
    // SearchQuery { query: SearchQuerySpec }
    #[serde(rename_all = "camelCase")]
    Substring { index: usize, length: Option<usize> },
    #[serde(rename_all = "camelCase")]
    Strlen,
    #[serde(rename_all = "camelCase")]
    TimeFormat { format: Option<String>, time_zone: Option<String>, locale: Option<String>, granularity: Option<Granularity>, as_millis: bool },
    #[serde(rename_all = "camelCase")]
    Time { time_format: String, result_format: String, joda: bool },
    #[serde(rename_all = "camelCase")]
    Javascript { function: String },
    // RegisteredLookup { lookup: Lookup, retain_missing_value: bool }
    // Lookup { lookup: Lookup, retain_missing_value: bool, injective: bool, replace_missing_value_wiht: String },

    #[serde(rename_all = "camelCase")]
    Cascade { extraction_fns: Vec<ExtractFN> },
    #[serde(rename_all = "camelCase")]
    StringFormat {format: String, null_handling: Option<NullHandling>},

    #[serde(rename_all = "camelCase")]
    Upper { locale: Option<String> },
    //todo
    #[serde(rename_all = "camelCase")]
    Lower { locale: Option<String> },

    #[serde(rename_all = "camelCase")]
    Bucket { size: usize, offset: usize },
}

#[serde(rename_all = "camelCase")]
#[derive(Deserialize, Serialize, Debug)]
pub enum NullHandling {
    NullString,
    EmptyString,
    ReturnNull,
}
// pub enum Interval {

// }
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "UPPERCASE")]
pub enum JoinType {
    Inner,
    Left,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "queryType")]
#[serde(rename_all = "camelCase")]
pub enum Query {
    #[serde(rename_all = "camelCase")]
    TopN {
        // todo: data_source would result in weird error message
        data_source: DataSource,
        dimension: Dimension,
        threshold: usize,
        metric: String,
        aggregations: Vec<Aggregation>,
        intervals: Vec<String>,
        granularity: Granularity,
    },
    #[serde(rename_all = "camelCase")]
    Scan {
        data_source: DataSource,
        intervals: Vec<String>,
        result_format: ResultFormat,
        filter: Option<Filter>,
        columns: Vec<String>,
        batch_size: usize,
        limit: Option<usize>,
        ordering: Option<Ordering>,
        context: std::collections::HashMap<String, String>,
    },
    #[serde(rename_all = "camelCase")]
    GroupBy {
        data_source: DataSource,
        dimensions: Vec<Dimension>,
        limit: LimitSpec,
        having: HavingSpec,
        granularity: Granularity,
        filter: Filter,
        aggregations: Vec<Aggregation>,
        post_aggregations: Vec<PostAggregation>,
        intervals: Vec<String>,
        subtotal_spec: Vec<Vec<String>>,
        context: std::collections::HashMap<String, String>,
    },
    #[serde(rename_all = "camelCase")]
    Search {
        data_source: DataSource,
        granularity: Granularity,
        filter: Filter,
        limit: usize,
        intervals: Vec<String>,
        search_dimensions: Vec<String>,
        query: SearchQuerySpec,
        sort: SortingOrder,
        context: std::collections::HashMap<String, String>,
    },
    #[serde(rename_all = "camelCase")]
    TimeBoundary {
        data_source: DataSource,
        bound: TimeBoundType,
        filter: Filter,
        context: std::collections::HashMap<String, String>,

    },
    #[serde(rename_all = "camelCase")]
    SegmentMetadata {
        data_source: DataSource,
        intervals: Vec<String>,
        to_include: String, 
        merge: bool,
        analysis_types: Vec<AnalysisType>,
        lenient_aggregator_merge: bool,
    },
    #[serde(rename_all = "camelCase")]
    DataSourceMetadata {
        data_source: DataSource,
        context: std::collections::HashMap<String, String>,
    }
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
    Rollup
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum TimeBoundType {
    MaxTime, MinTime
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum SearchQuerySpec {
    #[serde(rename_all = "camelCase")]
    InsensitiveContains { value : String },
    #[serde(rename_all = "camelCase")]
    Fragment { case_sensitive: bool, values: Vec<String>},
    #[serde(rename_all = "camelCase")]
    Contains {  case_sensitive: bool, value: String },
    #[serde(rename_all = "camelCase")]
    Regex { pattern: String },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum PostAggregation {
    #[serde(rename_all = "camelCase")]
    Arithmetic {
        name: String, 
        Fn: String,
        fields: Vec<PostAggregator>,
        ordering: String
    },
    DoubleGreatest {
        name: String,
        fields: Vec<PostAggregation>,
    },
    LongGreatest {
        name: String,
        fields: Vec<PostAggregation>,
    },
    LongLeast {
        name: String,
        fields: Vec<PostAggregation>,
    },
    DoubleLeast {
        name: String,
        fields: Vec<PostAggregation>,
    },
    #[serde(rename_all = "camelCase")]
    Javascript {
        name: String, 
        field_names: Vec<String>,
        function: String,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum PostAggregator {
    #[serde(rename_all = "camelCase")]
    FieldAccess {
        name: String,
        field_name: String,
    },
    #[serde(rename_all = "camelCase")]
    FinalizingFieldAccess {
        name: String,
        field_name: String,
    },
    #[serde(rename_all = "camelCase")]
    Constant {
        name: String,
        value: usize,
    },
    #[serde(rename_all = "camelCase")]
    HyperUniqueCardinality {
        field_name: String,
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LimitSpec {
    pub limit: usize,
    pub columns: Vec<OrderByColumnSpec>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OrderByColumnSpec {
    pub dimension: Dimension,
    pub direction: Ordering,
    pub dimension_order: SortingOrder,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum SortingOrder {
    Lexicographic,
    Alphanumeric,
    Strlen,
    Numeric,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ResultFormat {
    List,
    CompactedList,
    ValueVector,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Ordering {
    Ascending,
    Descending,
    None,
}

#[rustfmt::skip]
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum HavingSpec {
    Filter { filter: Filter},
    GreaterThan { aggregation: String, value: usize },
    EqualTo { aggregation: String, value: usize },
    LessThan { aggregation: String, value: usize },
    DimSelector { dimension: Dimension, value: usize }, //todo
    // DimSelector { dimension: Dimension, value: dyn std::fmt::Debug + Serialize + DeserializeOwned  },
    #[serde(rename_all = "camelCase")]
    And { having_specs: Vec<HavingSpec> },
    #[serde(rename_all = "camelCase")]
    Or { having_specs: Vec<HavingSpec> },
    #[serde(rename_all = "camelCase")]
    Not { having_specs: Box<HavingSpec> },
}
