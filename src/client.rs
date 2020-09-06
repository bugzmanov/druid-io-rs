use crate::query::model::{DataSourceMetadata, Query, JsonAny};
use crate::query::DataSource;
use crate::query::{
    group_by::GroupBy, scan::Scan, search::Search, time_boundary::TimeBoundary, Granularity, segment_metadata::SegmentMetadata, top_n::TopN,
};
use crate::serialization::default_for_null;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Deserialize, Serialize, Debug)]
pub struct DruidListResponse<T: DeserializeOwned + std::fmt::Debug + Serialize> {
    pub timestamp: String,
    #[serde(bound = "")]
    pub result: Vec<T>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MetadataResponse<T: DeserializeOwned + std::fmt::Debug + Serialize> {
    pub timestamp: String,
    #[serde(bound = "")]
    pub result: T,
}

type TopNResponse<T> = DruidListResponse<T>;

#[derive(Deserialize, Serialize, Debug)]
pub struct GroupByResponse<T: DeserializeOwned + std::fmt::Debug + Serialize> {
    pub timestamp: String,
    #[serde(bound = "")]
    pub event: T,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DimValue {
    pub dimension: String,
    pub value: JsonAny,
    pub count: usize,
}

type SearchResponse = DruidListResponse<DimValue>;

#[serde(rename_all = "camelCase")]
#[derive(Deserialize, Serialize, Debug)]
pub struct ScanResponse<T: DeserializeOwned + std::fmt::Debug + Serialize> {
    segment_id: String,
    columns: Vec<String>,
    #[serde(bound = "")]
    events: Vec<T>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MinMaxTime {
    pub max_time: Option<String>,
    pub min_time: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TimeBoundaryResponse {
    timestamp: String,
    result: MinMaxTime,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ColumnDefinition {
    #[serde(rename (deserialize = "type"))]
    column_type: String,
    has_multiple_values: bool,
    size: usize,
    cardinality: Option<f32>,
    min_value: Option<JsonAny>,
    max_value: Option<JsonAny>,
    error_message: Option<String>

}
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AggregatorDefinition {
    #[serde(rename (deserialize = "type"))]
    aggr_type: String,
    name: String,
    field_name: String,
    expression: Option<String>
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TimestampSpec {
    column: String,
    format: String,
    missing_value: Option<String>,
}
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SegmentMetadataResponse {
    id: String,
    #[serde(default, deserialize_with = "default_for_null")]
    intervals: Vec<String>,
    columns: HashMap<String, ColumnDefinition>,
    query_granularity: Option<String>,
    rollup: Option<bool>,
    size: Option<usize>,
    num_rows: Option<usize>,
    timestamp_spec: TimestampSpec,
    #[serde(default, deserialize_with = "default_for_null")]
    aggregators: HashMap<String, AggregatorDefinition>,
}
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum DruidClientError {
    #[error("http connection error")]
    HttpConnection { source: reqwest::Error },
    #[error("the data for key `{0}` is not available")]
    Redaction(String),
    #[error("invalid header (expected {expected:?}, found {found:?})")]
    InvalidHeader { expected: String, found: String },
    #[error("couldn't serialize object to json")]
    ParsingError { source: serde_json::Error },
    #[error("couldn't deserialize json to object")]
    ParsingResponseError { source: serde_json::Error }, // todo: original json but with manageable size
    #[error("Server responded with an error")]
    ServerError { response: String },
    #[error("unknown data store error")]
    Unknown,
}
pub struct DruidClient {
    http_client: Client,
    nodes: Vec<String>,
}

type ClientResult<T> = Result<T, DruidClientError>;

impl DruidClient {
    pub fn new(nodes: &Vec<String>) -> Self {
        DruidClient {
            http_client: Client::new(),
            nodes: nodes.clone(),
        }
    }

    fn url(&self) -> &str {
        "http://localhost:8888/druid/v2/?pretty"
    }

    async fn http_query(&self, request: &str) -> Result<String, DruidClientError> {
        let response_str = self
            .http_client
            .post(self.url())
            .body(request.to_string())
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .send()
            .await
            .map_err(|source| DruidClientError::HttpConnection { source: source })?
            .text()
            .await
            .map_err(|source| DruidClientError::HttpConnection { source: source })?;

        let json_value = serde_json::from_str::<serde_json::Value>(&response_str)
            .map_err(|err| DruidClientError::ParsingError { source: err });
        if let Some(_) = json_value?.get("error") {
            return Err(DruidClientError::ServerError {
                response: response_str,
            });
        }
        Ok(response_str)
    }

    pub async fn query<'a, T: DeserializeOwned + std::fmt::Debug + Serialize>(
        &self,
        query: &Query,
    ) -> Result<Vec<T>, DruidClientError> {
        self._query(query).await
    }
    pub async fn top_n<'a, T: DeserializeOwned + std::fmt::Debug + Serialize>(
        &self,
        query: &TopN,
    ) -> ClientResult<Vec<TopNResponse<T>>> {
        self._query(query).await
    }

    pub async fn search<'a, T: DeserializeOwned + std::fmt::Debug + Serialize>(
        &self,
        query: &Search,
    ) -> ClientResult<Vec<SearchResponse>> {
        self._query(query).await
    }

    pub async fn group_by<'a, T: DeserializeOwned + std::fmt::Debug + Serialize>(
        &self,
        query: &GroupBy,
    ) -> ClientResult<Vec<GroupByResponse<T>>> {
        self._query(query).await
    }
    pub async fn scan<'a, T: DeserializeOwned + std::fmt::Debug + Serialize>(
        &self,
        query: &Scan,
    ) -> ClientResult<Vec<ScanResponse<T>>> {
        self._query(query).await
    }
    pub async fn time_boundary<'a, T: DeserializeOwned + std::fmt::Debug + Serialize>(
        &self,
        query: &TimeBoundary,
    ) -> ClientResult<Vec<TimeBoundaryResponse>> {
        self._query(query).await
    }
    pub async fn segment_metadata(
        &self,
        query: &SegmentMetadata,
    ) -> ClientResult<Vec<SegmentMetadataResponse>> {
        self._query(query).await
    }

    async fn _query<Req, Resp>(&self, query: &Req) -> ClientResult<Resp>
    where
        Req: Serialize,
        Resp: DeserializeOwned,
    {
        let request = serde_json::to_string(&query)
            .map_err(|err| DruidClientError::ParsingError { source: err });

        let response = match request {
            Ok(str) => self.http_query(&str).await,
            Err(e) => Err(e),
        };

        let response = dbg!(response).and_then(|str| {
            serde_json::from_str::<Resp>(&str)
                .map_err(|source| DruidClientError::ParsingResponseError { source: source })
        });

        response
    }

    pub async fn datasource_metadata(
        self,
        data_source: DataSource,
    ) -> ClientResult<Vec<MetadataResponse<HashMap<String, String>>>> {
        let query = DataSourceMetadata {
            data_source: data_source,
            context: Default::default(),
        };

        self._query(&query).await
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::query::{
        group_by::{
            GroupBy, GroupByBuilder, HavingSpec, LimitSpec, OrderByColumnSpec, PostAggregation,
            PostAggregator,
        },
        scan::{ResultFormat, Scan},
        search::SearchQuerySpec,
        segment_metadata::{AnalysisType, SegmentMetadata, ToInclude},
        time_boundary::{TimeBoundType, TimeBoundary},
        Filter, JoinType, Ordering, OutputType, SortingOrder, top_n::TopN, model::Aggregation, Dimension,
    };
    #[derive(Serialize, Deserialize, Debug)]
    struct WikiPage {
        page: String,
        user: Option<String>,
        count: usize,
    }

    #[test]
    fn test_top_n_query() {
        let mut context = HashMap::new();
        context.insert("resultAsArray".to_string(), "true".to_string());
        let top_n = TopN {
            data_source: DataSource::table("wikipedia"),
            dimension: Dimension::default("page"),
            threshold: 10,
            metric: "count".into(),
            aggregations: vec![
                Aggregation::count("count"),
                Aggregation::StringFirst {
                    name: "user".into(),
                    field_name: "user".into(),
                    max_string_bytes: 1024,
                },
            ],
            intervals: vec!["-146136543-09-08T08:23:32.096Z/146140482-04-24T15:36:27.903Z".into()],
            granularity: Granularity::All,
            context: context,
        };
        let druid_client = DruidClient::new(&vec!["ololo".into()]);
        let result = tokio_test::block_on(druid_client.top_n::<WikiPage>(&top_n));
        println!("{:?}", result.unwrap());
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    struct ScanEvent {
        #[serde(rename(deserialize = "__time"))]
        time: usize,
        city_name: Option<String>,
        comment: Option<String>,
        namespace: Option<String>,
        page: Option<String>,
        region_iso_code: Option<String>,
        user: String,

        #[serde(rename(deserialize = "c.languages"))]
        languages: Option<String>,
        count: usize,
    }
    #[test]
    fn test_scan_join() {
        let scan = Scan {
            data_source: DataSource::join(JoinType::Inner)
                .left(DataSource::table("wikipedia"))
                .right(
                    DataSource::query(
                        Scan {
                            data_source: DataSource::table("countries"),
                            batch_size: 10,
                            intervals: vec![
                                "-146136543-09-08T08:23:32.096Z/146140482-04-24T15:36:27.903Z"
                                    .into(),
                            ],
                            result_format: ResultFormat::List,
                            columns: vec!["Name".into(), "languages".into()],
                            limit: None,
                            filter: None,
                            ordering: Some(Ordering::None),
                            context: std::collections::HashMap::new(),
                        }
                        .into(),
                    ),
                    "c.",
                )
                .condition("countryName == \"c.Name\"")
                .build()
                .unwrap(),
            batch_size: 10,
            intervals: vec!["-146136543-09-08T08:23:32.096Z/146140482-04-24T15:36:27.903Z".into()],
            result_format: ResultFormat::List,
            columns: vec![],
            limit: Some(10),
            filter: None,
            ordering: Some(Ordering::None),
            context: std::collections::HashMap::new(),
        };

        let druid_client = DruidClient::new(&vec!["ololo".into()]);
        let result = tokio_test::block_on(druid_client.scan::<ScanEvent>(&scan));
        println!("{:?}", result.unwrap());
    }
    #[test]
    fn test_group_by() {
        let group_by = GroupBy {
            data_source: DataSource::table("wikipedia"),
            dimensions: vec![Dimension::Default {
                dimension: "page".into(),
                output_name: "page".into(),
                output_type: OutputType::STRING,
            }],
            limit_spec: Some(LimitSpec {
                limit: 10,
                columns: vec![OrderByColumnSpec::new(
                    "page",
                    Ordering::Descending,
                    SortingOrder::Alphanumeric,
                )],
            }),
            having: Some(HavingSpec::greater_than("count_ololo", 0.01.into())),
            granularity: Granularity::All,
            filter: Some(Filter::selector("user", "Taffe316")),
            aggregations: vec![
                Aggregation::count("count"),
                Aggregation::StringFirst {
                    name: "user".into(),
                    field_name: "user".into(),
                    max_string_bytes: 1024,
                },
            ],
            post_aggregations: vec![PostAggregation::Arithmetic {
                name: "count_ololo".into(),
                function: "/".into(),
                fields: vec![
                    PostAggregator::field_access("count_percent", "count"),
                    PostAggregator::constant("hundred", 100.into()),
                ],
                ordering: None,
            }],
            intervals: vec!["-146136543-09-08T08:23:32.096Z/146140482-04-24T15:36:27.903Z".into()],
            subtotal_spec: Default::default(),
            context: Default::default(),
        };
        let druid_client = DruidClient::new(&vec!["ololo".into()]);
        let result = tokio_test::block_on(druid_client.group_by::<WikiPage>(&group_by));
        println!("{:?}", result.unwrap());
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Page {
        count: usize,
        count_ololo: f32,
        title: String,
        user: String,
    }
    #[test]
    fn test_group_by_builder() {
        let group_by = GroupByBuilder::new(DataSource::table("wikipedia"))
            .dimensions(vec![Dimension::Default {
                dimension: "page".into(),
                output_name: "title".into(),
                output_type: OutputType::STRING,
            }])
            .limit(LimitSpec {
                limit: 10,
                columns: vec![OrderByColumnSpec::new(
                    "title",
                    Ordering::Descending,
                    SortingOrder::Alphanumeric,
                )],
            })
            .having(HavingSpec::greater_than("count_ololo", 0.001.into()))
            .filter(Filter::selector("user", "Taffe316"))
            .aggregations(vec![
                Aggregation::count("count"),
                Aggregation::StringFirst {
                    name: "user".into(),
                    field_name: "user".into(),
                    max_string_bytes: 1024,
                },
            ])
            .post_aggregations(vec![PostAggregation::Arithmetic {
                name: "count_ololo".into(),
                function: "/".into(),
                fields: vec![
                    PostAggregator::field_access("count_percent", "count"),
                    PostAggregator::constant("hundred", 100.into()),
                ],
                ordering: None,
            }])
            .intervals(vec![
                "-146136543-09-08T08:23:32.096Z/146140482-04-24T15:36:27.903Z".into(),
            ])
            .add_context("groupByStrategy", "v2")
            .add_context("resultAsArray", "true")
            .build();
        let druid_client = DruidClient::new(&vec!["ololo".into()]);
        let result = tokio_test::block_on(druid_client.group_by::<Page>(&group_by));
        println!("{:?}", result.unwrap());
    }

    #[test]
    fn test_search() {
        let search = Search {
            data_source: DataSource::table("wikipedia"),
            search_dimensions: vec!["page".into(), "user".into()],
            query: SearchQuerySpec::contains_insensitive("500"),
            sort: None,
            filter: None,
            limit: 20,
            intervals: vec!["-146136543-09-08T08:23:32.096Z/146140482-04-24T15:36:27.903Z".into()],
            context: Default::default(),
            granularity: Granularity::All,
        };
        let druid_client = DruidClient::new(&vec!["ololo".into()]);
        let result = tokio_test::block_on(druid_client.search::<WikiPage>(&search));
        println!("{:?}", result.unwrap());
    }
    #[test]
    fn test_time_boundary() {
        let top_n = TimeBoundary {
            data_source: DataSource::table("wikipedia"),
            filter: None,
            context: Default::default(),
            bound: TimeBoundType::MinMaxTime,
        };
        let druid_client = DruidClient::new(&vec!["ololo".into()]);
        let result = tokio_test::block_on(druid_client.time_boundary::<WikiPage>(&top_n));
        println!("{:?}", result.unwrap());
    }
    #[test]
    fn test_data_source_metadata() {
        let druid_client = DruidClient::new(&vec!["ololo".into()]);
        let result =
            tokio_test::block_on(druid_client.datasource_metadata(DataSource::table("wikipedia")));
        println!("{:?}", result.unwrap());
    }
    #[test]
    fn test_segment_metadata() {
        let segment_query = SegmentMetadata {
            data_source: DataSource::table("wikipedia"),
            intervals: vec!["-146136543-09-08T08:23:32.096Z/146140482-04-24T15:36:27.903Z".into()],
            to_include: ToInclude::All,
            merge: false,
            analysis_types: vec![
                AnalysisType::Minmax,
                AnalysisType::Size,
                AnalysisType::Interval,
                AnalysisType::TimestampSpec,
                AnalysisType::QueryGranularity,
                AnalysisType::Aggregators,
                AnalysisType::Rollup,
                AnalysisType::Cardinality,
            ],
            lenient_aggregator_merge: false,
        };

        let druid_client = DruidClient::new(&vec!["ololo".into()]);
        let result = tokio_test::block_on(
            druid_client.segment_metadata( &segment_query)
        );
        println!("{:?}", result.unwrap());
    }
}
