use super::{
    model::{Aggregation, JsonAny, JsonNumber},
    SortingOrder,
};
use crate::query::DataSource;
use crate::query::Dimension;
use crate::query::Filter;
use crate::query::Granularity;
use crate::query::Ordering;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "queryType", rename = "groupBy")]
pub struct GroupBy {
    pub data_source: DataSource,
    pub dimensions: Vec<Dimension>,
    pub limit_spec: Option<LimitSpec>,
    pub having: Option<HavingSpec>,
    pub granularity: Granularity,
    pub filter: Option<Filter>,
    pub aggregations: Vec<Aggregation>,
    pub post_aggregations: Vec<PostAggregation>,
    pub intervals: Vec<String>,
    pub subtotal_spec: Vec<Vec<String>>,
    pub context: std::collections::HashMap<String, String>,
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

impl OrderByColumnSpec {
    pub fn new(dimension: &str, direction: Ordering, dimension_order: SortingOrder) -> Self {
        OrderByColumnSpec {
            dimension: dimension.to_string(),
            direction: direction,
            dimension_order: dimension_order,
        }
    }
}

#[rustfmt::skip]
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum HavingSpec {
    Filter { filter: Filter},
    GreaterThan { aggregation: String, value: JsonNumber },
    EqualTo { aggregation: String, value: JsonNumber },
    LessThan { aggregation: String, value: JsonNumber },
    DimSelector { dimension: Dimension, value: JsonAny }, //todo
    #[serde(rename_all = "camelCase")]
    And { having_specs: Vec<HavingSpec> },
    #[serde(rename_all = "camelCase")]
    Or { having_specs: Vec<HavingSpec> },
    #[serde(rename_all = "camelCase")]
    Not { having_specs: Box<HavingSpec> },
}

impl HavingSpec {
    pub fn filter(filter: Filter) -> Self {
        HavingSpec::Filter { filter: filter }
    }
    pub fn greater_than(aggregation: &str, value: JsonNumber) -> Self {
        HavingSpec::GreaterThan {
            aggregation: aggregation.to_string(),
            value: value,
        }
    }
    pub fn equal_to(aggregation: &str, value: JsonNumber) -> Self {
        HavingSpec::EqualTo {
            aggregation: aggregation.to_string(),
            value: value,
        }
    }
    pub fn less_than(aggregation: &str, value: JsonNumber) -> Self {
        HavingSpec::LessThan {
            aggregation: aggregation.to_string(),
            value: value,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum PostAggregation {
    #[serde(rename_all = "camelCase")]
    Arithmetic {
        name: String,
        #[serde(rename(serialize = "fn"))]
        function: String,
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
    FieldAccess { name: String, field_name: String },
    #[serde(rename_all = "camelCase")]
    FinalizingFieldAccess { name: String, field_name: String },
    #[serde(rename_all = "camelCase")]
    Constant { name: String, value: JsonAny },
    #[serde(rename_all = "camelCase")]
    HyperUniqueCardinality { field_name: String },
}

impl PostAggregator {
    pub fn field_access(name: &str, field_name: &str) -> Self {
        PostAggregator::FieldAccess {
            name: name.to_string(),
            field_name: field_name.to_string(),
        }
    }
    pub fn finalized_field_access(name: &str, field_name: &str) -> Self {
        PostAggregator::FinalizingFieldAccess {
            name: name.to_string(),
            field_name: field_name.to_string(),
        }
    }
    pub fn constant(name: &str, value: JsonAny) -> Self {
        PostAggregator::Constant {
            name: name.to_string(),
            value: value,
        }
    }
    pub fn hyper_unique_cardinality(field_name: &str) -> Self {
        PostAggregator::HyperUniqueCardinality {
            field_name: field_name.to_string(),
        }
    }
}

pub struct GroupByBuilder {
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
}

impl GroupByBuilder {
    pub fn new(data_source: DataSource) -> Self {
        GroupByBuilder {
            data_source: data_source,
            dimensions: vec![],
            limit_spec: None,
            having: None,
            granularity: Granularity::All,
            filter: None,
            aggregations: vec![],
            post_aggregations: vec![],
            intervals: vec![],
            subtotal_spec: vec![],
            context: std::collections::HashMap::new(),
        }
    }
    pub fn dimensions(mut self, dimensions: Vec<Dimension>) -> Self {
        self.dimensions = dimensions;
        self
    }
    pub fn limit(mut self, limit: LimitSpec) -> Self {
        self.limit_spec = Some(limit);
        self
    }
    pub fn having(mut self, having: HavingSpec) -> Self {
        self.having = Some(having);
        self
    }
    pub fn granularity(mut self, granularity: Granularity) -> Self {
        self.granularity = granularity;
        self
    }
    pub fn filter(mut self, filter: Filter) -> Self {
        self.filter = Some(filter);
        self
    }
    pub fn aggregations(mut self, aggr: Vec<Aggregation>) -> Self {
        self.aggregations = aggr;
        self
    }
    pub fn post_aggregations(mut self, aggr: Vec<PostAggregation>) -> Self {
        self.post_aggregations = aggr;
        self
    }
    pub fn intervals(mut self, intervals: Vec<&str>) -> Self {
        self.intervals = intervals.iter().map(|s| s.to_string()).collect();
        self
    }
    pub fn subtotal_spec(mut self, subtotals: Vec<Vec<String>>) -> Self {
        self.subtotal_spec = subtotals;
        self
    }
    pub fn context(mut self, context: std::collections::HashMap<String, String>) -> Self {
        self.context = context;
        self
    }

    pub fn add_context(mut self, key: &str, value: &str) -> Self {
        self.context.insert(key.to_string(), value.to_string());
        self
    }
    pub fn build(self) -> GroupBy {
        GroupBy {
            data_source: self.data_source,
            dimensions: self.dimensions,
            limit_spec: self.limit_spec,
            having: self.having,
            granularity: self.granularity,
            filter: self.filter,
            aggregations: self.aggregations,
            post_aggregations: self.post_aggregations,
            intervals: self.intervals,
            subtotal_spec: self.subtotal_spec,
            context: self.context,
        }
    }
}
