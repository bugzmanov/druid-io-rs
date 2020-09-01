use crate::query::Ordering;
use crate::query::DataSource;
use crate::query::Filter;
use crate::query::Granularity;
use crate::query::Dimension;
use serde::{Deserialize, Serialize};
use super::SortingOrder;


// }
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
        limit_spec: Option<LimitSpec>,
        having: Option<HavingSpec>,
        granularity: Granularity,
        filter: Option<Filter>,
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
#[serde(tag = "type")]
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
#[serde(tag = "type")]
pub enum PostAggregation {
    #[serde(rename_all = "camelCase")]
    Arithmetic {
        name: String, 
        Fn: String,
        fields: Vec<PostAggregator>,
        ordering: Option<String>,
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
#[serde(tag = "type")]
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
#[serde(tag = "type", rename = "default")]
pub struct LimitSpec {
    pub limit: usize,
    pub columns: Vec<OrderByColumnSpec>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OrderByColumnSpec {
    pub dimension: String,
    pub direction: Ordering,
    pub dimension_order: SortingOrder,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ResultFormat {
    List,
    CompactedList,
    ValueVector,
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
