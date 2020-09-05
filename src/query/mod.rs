use serde::{Deserialize, Serialize};
use model::Query;

pub mod model;
pub mod group_by;
pub mod search;
pub mod scan;
pub mod time_boundary;
pub mod segment_metadata;


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
    #[serde(rename_all = "camelCase")]
    ListFiltered {
        delegate: Box<Dimension>,
        values: Vec<String>,
        is_whitelist: bool,
    },

    #[serde(rename_all = "camelCase")]
    RegexFiltered {
        delegate: Box<Dimension>,
        pattern: String,
    },
    #[serde(rename_all = "camelCase")]
    PrefixFiltered {
        delegate: Box<Dimension>,
        prefix: String,
    },
    #[serde(rename_all = "camelCase")]
    #[serde(rename(serialize = "lookup"))]
    LookupMap {
        dimension: String,
        output_name: String,
        replace_missing_value_with: String,
        retain_missing_value: bool,
        lookup: LookupMap,
    },

    Lookup {
        dimension: String,
        output_name: String,
        name: String,
    },
}

impl Dimension {
    pub fn default(dimension: &str) -> Dimension {
        Dimension::Default {
            dimension: dimension.into(),
            output_name: dimension.into(),
            output_type: OutputType::STRING,
        }
    }

    pub fn regex(dimension: Dimension, pattern: &str) -> Dimension {
        Dimension::RegexFiltered {
            pattern: pattern.into(),
            delegate: Box::new(dimension),
        }
    }
    pub fn prefix(dimension: Dimension, prefix: &str) -> Dimension {
        Dimension::PrefixFiltered {
            prefix: prefix.into(),
            delegate: Box::new(dimension),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type", rename = "map")]
pub struct LookupMap {
    map: std::collections::HashMap<String, String>,
    is_one_to_one: bool,
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
    Duration { duration: usize },
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
    #[serde(rename_all = "camelCase")]
    RegisteredLookup { lookup: String, retain_missing_value: bool },
    #[serde(rename_all = "camelCase")]
    Lookup { lookup: LookupMap, retain_missing_value: bool, injective: bool, replace_missing_value_with: String },

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
    ColumnComparison {
        dimensions: Vec<String>,
    },
    Regex {
        dimension: String,
        pattern: String,
    },
    And {
        fields: Vec<Filter>,
    },
    Or {
        fields: Vec<Filter>,
    },
    Not {
        field: Box<Filter>,
    },
    Javascript {
        dimension: String,
        function: String,
    },
    Search {
        dimension: String,
        query: FilterQuerySpec,
    },
    In {
        dimension: String,
        values: Vec<String>,
    },
    #[serde(rename_all = "camelCase")]
    Like {
        dimension: String,
        pattern: String,
        escape: Option<String>,
        extraction_fn: Option<ExtractFN>,
    },
    #[serde(rename_all = "camelCase")]
    Bound {
        dimension: String,
        lower: String,
        upper: String,
        lower_strict: bool,
        upper_strict: bool,
        ordering: SortingOrder,
        extraction_fn: Option<ExtractFN>,
    },
    #[serde(rename_all = "camelCase")]
    Interval {
        dimension: String,
        intervals: Vec<String>,
        extraction_fn: Option<ExtractFN>,
    },
    True,
}

impl Filter {
    pub fn selector(dimension: &str, value: &str) -> Filter {
        Filter::Selector {
            dimension: dimension.to_string(),
            value: value.to_string(),
            extract_fn: None
        }
    }

    pub fn column_comparison(dimensions: Vec<&str>) -> Self {
        Filter::ColumnComparison {
            dimensions: dimensions.iter().map(|s| s.to_string()).collect()
        }
    }

    pub fn regex(dimension: &str, pattern: &str) -> Self {
        Filter::Regex {
            dimension: dimension.to_string(),
            pattern: pattern.to_string(),
        }
    }

    pub fn javascript(dimension:& str, javascript: &str) -> Self {
        Filter::Javascript {
            dimension: dimension.to_string(),
            function: javascript.to_string(),
        }
    }

    pub fn in_values(dimension: &str, values: Vec<&str>) -> Self {
        Filter::In {
            dimension: dimension.to_string(),
            values: values.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn like(dimension: &str, pattern: &str) -> Self {
        Filter::Like {
            dimension: dimension.to_string(),
            pattern: pattern.to_string(),
            escape: None,
            extraction_fn: None,
            
        }
    }
}
#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum FilterQuerySpec {
    #[serde(rename_all = "camelCase")]
    Contains { value: String, case_sensitive: bool },
    #[serde(rename_all = "camelCase")]
    InsensitiveContains { value: String },
    #[serde(rename_all = "camelCase")]
    Fragment {
        values: Vec<String>,
        case_sensitive: bool,
    },
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
pub enum Ordering {
    Ascending,
    Descending,
    None,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum SortingOrder {
    Lexicographic,
    Alphanumeric,
    Strlen,
    Numeric,
}
