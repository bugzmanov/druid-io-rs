use crate::query::model::Aggregation;
use crate::query::model::{DataSourceMetadata, Query};
use crate::query::DataSource;
use crate::query::Dimension;
use crate::query::Granularity;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Deserialize, Serialize, Debug)]
pub struct QueryListResult<T: DeserializeOwned + std::fmt::Debug + Serialize> {
    timestamp: String,
    #[serde(bound = "")]
    result: Vec<T>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct QueryResult<T: DeserializeOwned + std::fmt::Debug + Serialize> {
    timestamp: String,
    #[serde(bound = "")]
    result: T,
}

#[derive(Error, Debug)]
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
    ParsingResponseError { source: serde_json::Error },
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

    pub fn url(&self) -> &str {
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
    ) -> Result<Vec<QueryListResult<T>>, DruidClientError> {
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

        let response = response.and_then(|str| {
            serde_json::from_str::<Resp>(&str)
                .map_err(|source| DruidClientError::ParsingResponseError { source: source })
        });

        response
    }

    pub async fn datasource_metadata(
        self,
        data_source: DataSource,
    ) -> ClientResult<Vec<QueryResult<HashMap<String, String>>>> {
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
    use crate::query::model::OrderByColumnSpec;
    use crate::query::{
        model::{
            GroupByBuilder, HavingSpec, LimitSpec, PostAggregation, PostAggregator, ResultFormat,
            SearchQuerySpec, ToInclude,
        },
        Filter, JoinType, Ordering, OutputType, SortingOrder,
    };
    #[derive(Serialize, Deserialize, Debug)]
    struct WikiPage {
        page: String,
        user: Option<String>,
        count: usize,
    }

    #[test]
    fn test_top_n_query() {
        let top_n = Query::TopN {
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
        };
        let druid_client = DruidClient::new(&vec!["ololo".into()]);
        let result = tokio_test::block_on(druid_client.query::<WikiPage>(&top_n));
        println!("{:?}", result.unwrap());
    }

    #[test]
    fn test_scan_join() {
        let scan = Query::Scan {
            data_source: DataSource::join(JoinType::Inner)
                .left(DataSource::table("wikipedia"))
                .right(
                    DataSource::query(Query::Scan {
                        data_source: DataSource::table("countries"),
                        batch_size: 10,
                        intervals: vec![
                            "-146136543-09-08T08:23:32.096Z/146140482-04-24T15:36:27.903Z".into(),
                        ],
                        result_format: ResultFormat::List,
                        columns: vec!["Name".into(), "languages".into()],
                        limit: None,
                        filter: None,
                        ordering: Some(Ordering::None),
                        context: std::collections::HashMap::new(),
                    }),
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
        let result = tokio_test::block_on(druid_client.query::<WikiPage>(&scan));
        println!("{:?}", result.unwrap());
    }
    #[test]
    fn test_group_by() {
        let group_by = Query::GroupBy {
            data_source: DataSource::table("wikipedia"),
            dimensions: vec![Dimension::Default {
                dimension: "page".into(),
                output_name: "title".into(),
                output_type: OutputType::STRING,
            }],
            limit_spec: Some(LimitSpec {
                limit: 10,
                columns: vec![OrderByColumnSpec::new(
                    "title",
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
                Fn: "/".into(),
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
        let result = tokio_test::block_on(druid_client.query::<WikiPage>(&group_by));
        println!("{:?}", result.unwrap());
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
            .having(HavingSpec::greater_than("count_ololo", 0.01.into()))
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
                Fn: "/".into(),
                fields: vec![
                    PostAggregator::field_access("count_percent", "count"),
                    PostAggregator::constant("hundred", 100.into()),
                ],
                ordering: None,
            }])
            .intervals(vec![
                "-146136543-09-08T08:23:32.096Z/146140482-04-24T15:36:27.903Z".into(),
            ])
            .build();
        let druid_client = DruidClient::new(&vec!["ololo".into()]);
        let result = tokio_test::block_on(druid_client.query::<WikiPage>(&group_by));
        println!("{:?}", result.unwrap());
    }

    #[test]
    fn test_search() {
        let top_n = Query::Search {
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
        let result = tokio_test::block_on(druid_client.query::<WikiPage>(&top_n));
        println!("{:?}", result.unwrap());
    }
    #[test]
    fn test_time_boundary() {
        let top_n = Query::TimeBoundary {
            data_source: DataSource::table("wikipedia"),
            filter: None,
            context: Default::default(),
            bound: None,
        };
        let druid_client = DruidClient::new(&vec!["ololo".into()]);
        let result = tokio_test::block_on(druid_client.query::<WikiPage>(&top_n));
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
        let top_n = Query::SegmentMetadata {
            data_source: DataSource::table("wikipedia"),
            intervals: vec!["-146136543-09-08T08:23:32.096Z/146140482-04-24T15:36:27.903Z".into()],
            to_include: ToInclude::All,
            merge: true,
            analysis_types: vec![],
            lenient_aggregator_merge: false,
        };

        let druid_client = DruidClient::new(&vec!["ololo".into()]);
        let result = tokio_test::block_on(
            druid_client.query::<std::collections::HashMap<String, String>>(&top_n),
        );
        println!("{:?}", result.unwrap());
    }
}
