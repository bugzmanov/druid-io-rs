use serde::{Deserialize, Serialize};

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

#[derive(Debug, Serialize, Deserialize)]
pub enum OutputType {
    STRING,
    LONG,
    FLOAT,
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
#[derive(Serialize, Deserialize, Debug)]
pub enum HllType {
    #[allow(non_camel_case_types)]
    HLL_4,
    #[allow(non_camel_case_types)]
    HLL_6,
    #[allow(non_camel_case_types)]
    HLL_8,
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Granularity {
    All,
    None,
    Second,
    Minute,
    FifteenMinute,
    ThirtyMinute,
    Hour,
    Day,
    Week,
    Month,
    Quarter,
    Year,
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
            extract_fn: None,
        }
    }

    pub fn column_comparison(dimensions: Vec<&str>) -> Self {
        Filter::ColumnComparison {
            dimensions: dimensions.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn regex(dimension: &str, pattern: &str) -> Self {
        Filter::Regex {
            dimension: dimension.to_string(),
            pattern: pattern.to_string(),
        }
    }

    pub fn javascript(dimension: &str, javascript: &str) -> Self {
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
