use crate::query::model::Dimension;
use crate::query::model::Query;
use crate::query::model::{Aggregation, DataSource, DruidClientError, Granularity};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct QueryResult<T: DeserializeOwned + std::fmt::Debug + Serialize> {
    // timestamp: String,
    #[serde(bound = "")]
    result: Vec<T>,
}

pub struct DruidClient {
    http_client: Client,
    nodes: Vec<String>,
}

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

    pub async fn test_query(&self) -> Result<String, DruidClientError> {
        let content = self
            .http_client
            .post(self.url())
            .body(
                r#"
                {
                    "queryType": "topN",
                    "dataSource": {
                        "type": "table",
                        "name": "wikipedia"
                    },
                    "dimension": {
                        "type": "default",
                        "dimension": "page",
                        "outputName": "d0",
                        "outputType": "STRING"
                    },
                    "metric": "a0",
                    "threshold": 10,
                    "intervals": ["-146136543-09-08T08:23:32.096Z/146140482-04-24T15:36:27.903Z"],
                    "granularity": "ALL",
                    "aggregations": [
                        {
                        "type": "count",
                        "name": "a0"
                        }
                    ]
                }
            "#,
            )
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .send()
            .await
            .map_err(|source| DruidClientError::HttpConnection { source: source })?
            .text()
            .await
            .map_err(|source| DruidClientError::HttpConnection { source: source })?;

        Ok(content)
    }

    async fn query_str(&self, query: &Query) -> Result<String, DruidClientError> {
        let request = serde_json::to_string(query)
            .map_err(|err| DruidClientError::ParsingError { source: err });

        let response = self
            .http_client
            .post(self.url())
            .body(request?.clone())
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .send()
            .await
            .map_err(|source| DruidClientError::HttpConnection { source: source })?
            .text()
            .await
            .map_err(|source| DruidClientError::HttpConnection { source: source })?;

        Ok(response)
    }

    pub async fn query<'a, T: DeserializeOwned + std::fmt::Debug + Serialize>(
        &self,
        query: &Query,
    ) -> Result<Vec<QueryResult<T>>, DruidClientError> {
        let response_str = dbg!(self.query_str(query).await?);
        let json_value = serde_json::from_str::<serde_json::Value>(&response_str)
            .map_err(|err| DruidClientError::ParsingError { source: err });
        if let Some(error) = json_value?.get("error") {
            return Err(DruidClientError::ServerError {
                response: response_str,
            });
        }
        let response = serde_json::from_str::<Vec<QueryResult<T>>>(&response_str)
            .map_err(|source| DruidClientError::ParsingError { source: source });

        response
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::query::model::{ResultFormat, OutputType, JoinType, Ordering};
    #[test]
    fn test_basic() {
        let druid_client = DruidClient::new(&vec!["ololo".into()]);
        let result = tokio_test::block_on(druid_client.test_query());
        println!("{}", result.unwrap());
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct WikiPage {
        title: String,
        user: Option<String>,
        count: usize,
    }

    #[test]
    fn test_top_n_query() {
        let top_n = Query::TopN {
            data_source: DataSource::Table {
                name: "wikipedia".into(),
            },
            dimension: Dimension::Default {
                dimension: "page".into(),
                output_name: "title".into(),
                output_type: OutputType::STRING,
            },
            threshold: 10,
            metric: "count".into(),
            aggregations: vec![
                Aggregation::Count {
                    name: "count".into(),
                    // name: "count".into(),
                },
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
        let top_n = Query::Scan {
            data_source: DataSource::Join {
                left:  Box::new(DataSource::Table {name : "wikipedia".into()}),
                right:  Box::new(DataSource::Query {
                   query: Box::new(Query::Scan {
                        data_source: DataSource::Table { name: "countries".into() },
                        batch_size:10,
                        intervals: vec!["-146136543-09-08T08:23:32.096Z/146140482-04-24T15:36:27.903Z".into()],
                        result_format: ResultFormat::List,
                        columns: vec!["Name".into(), "languages".into()],
                        limit: None,
                        filter: None,
                        ordering: Some(Ordering::None),
                        context: std::collections::HashMap::new(),
                   }) 
                }),
                right_prefix: "c.".into(),
                condition: "countryName == \"c.Name\"".into(),
                join_type: JoinType::Inner,
            },
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
        let result = tokio_test::block_on(druid_client.query::<WikiPage>(&top_n));
        println!("{:?}", result.unwrap());
    }
}
