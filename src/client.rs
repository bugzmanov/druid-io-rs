use crate::connection::{BrokersPool, SelectionStategy, StaticPool};
use crate::query::response::GroupByResponse;
use crate::query::response::MetadataResponse;
use crate::query::response::ScanResponse;
use crate::query::response::SearchResponse;
use crate::query::response::SegmentMetadataResponse;
use crate::query::response::TimeBoundaryResponse;
use crate::query::response::{TimeseriesResponse, TopNResponse};
use crate::query::timeseries::Timeseries;
use crate::query::{
    group_by::GroupBy, scan::Scan, search::Search, segment_metadata::SegmentMetadata,
    time_boundary::TimeBoundary, top_n::TopN, DataSource,
};
use crate::query::{DataSourceMetadata, Query};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use thiserror::Error;

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
type ClientResult<T> = Result<T, DruidClientError>;

pub struct DruidClient {
    http_client: Client,
    brokers_pool: Box<dyn BrokersPool>,
}

impl DruidClient {
    pub fn new(nodes: Vec<String>) -> Self {
        let strategy = SelectionStategy::default_for(&nodes);
        DruidClient {
            http_client: Client::new(),
            brokers_pool: Box::new(StaticPool::new(nodes, strategy)),
        }
    }

    fn url(&self) -> String {
        format!("http://{}/druid/v2/?pretty", self.brokers_pool.broker())
    }

    async fn http_query(&self, request: &str) -> Result<String, DruidClientError> {
        let response_str = self
            .http_client
            .post(&self.url())
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
    ) -> ClientResult<Vec<T>> {
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

    pub async fn timeseries<'a, T: DeserializeOwned + std::fmt::Debug + Serialize>(
        &self,
        query: &Timeseries,
    ) -> ClientResult<Vec<TimeseriesResponse<T>>> {
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
    ) -> ClientResult<Vec<MetadataResponse<HashMap<String, String>>>> {
        let query = DataSourceMetadata {
            data_source: data_source,
            context: Default::default(),
        };

        self._query(&query).await
    }
}
