use crate::model::Dimension;
use crate::model::{Aggregation, DataSource, DruidClientError, Granularity};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

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
}

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

    pub async fn query(&self) -> Result<String, DruidClientError> {
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

    pub async fn query2(&self, topN: &Query) -> Result<String, DruidClientError> {
        let request = serde_json::to_string(topN)
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

    pub async fn query3<'a, T: DeserializeOwned + std::fmt::Debug + Serialize>(
        &self,
        topN: &Query,
    ) -> Result<Vec<QueryResult<T>>, DruidClientError> {
        let response_str = self.query2(topN).await?;
        let response = serde_json::from_str::<Vec<QueryResult<T>>>(&response_str)
            .map_err(|source| DruidClientError::ParsingError { source: source });

        response
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::model::OutputType;
    #[test]
    fn test_basic() {
        let druid_client = DruidClient::new(&vec!["ololo".into()]);
        let result = tokio_test::block_on(druid_client.query());
        println!("{}", result.unwrap());
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct WikiPage {
        title: String,
        count: usize,
    }

    #[test]
    fn test_topN_query() {
        let top_n = Query::TopN {
            data_source: DataSource::Table {
                name: "wikipedia".into(),
            },
            dimension: Dimension::Default {
                dimension: "page".into(),
                outputName: "title".into(),
                outputType: OutputType::STRING,
            },
            threshold: 10,
            metric: "count".into(),
            aggregations: vec![Aggregation::Count {
                name: "count".into(),
            }],
            intervals: vec!["-146136543-09-08T08:23:32.096Z/146140482-04-24T15:36:27.903Z".into()],
            granularity: Granularity::ALL,
        };
        let druid_client = DruidClient::new(&vec!["ololo".into()]);
        let result = tokio_test::block_on(druid_client.query3::<WikiPage>(&top_n));
        println!("ololo {:?}", result.unwrap());
    }
}
