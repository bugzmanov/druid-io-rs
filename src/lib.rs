//! # Async rust client for Apache Druid
//!
//! Fully asynchronous, future-enabled [Apache Druid](http://druid.io/) client library for rust programming language.
//!
//! The library provides staticly typed API for [Native Queries](https://druid.apache.org/docs/latest/querying/querying.html)
//!
//! ## Installation
//! The library is hosted on [crates.io](https://crates.io/crates/druid-io/).
//!
//! ```toml
//! [dependencies]
//! druid-io = "*"
//! ```
//!
//! ## Supported Native Queries
//!
//! * Timeseries
//! * TopN
//! * GroupBy
//! * Scan
//! * Search
//! * TimeBoundary
//! * SegmentMetadata
//! * DataSourceMetadata
//!
//! ## Usage
//!
//! ### Client
//!
//! Connect to a druid cluster throughly staticly provided list of brokers:
//!
//! ```rust
//! use druid_io::client::DruidClient;
//!
//! let druid_client = DruidClient::new(vec!["localhost:8082".to_string()]);
//! ```
//!
//! ### Querying
//!
//! #### Timeseries
//!
//! See [Timeseries query documentation](https://druid.apache.org/docs/latest/querying/timeseriesquery.html)
//!
//! ```rust
//! use druid_io::client::DruidClient;
//! use serde::Deserialize;
//! use serde::Serialize;
//! use druid_io::{
//!     query::timeseries::Timeseries,
//!     query::{
//!         definitions::Aggregation,
//!         definitions::{Dimension, Filter, Granularity, Ordering, OutputType, SortingOrder},
//!         group_by::{
//!             PostAggregation, PostAggregator,
//!         },
//!         DataSource
//!     },
//! };
//! 
//! #[derive(Serialize, Deserialize, Debug)]
//! pub struct TimeAggr {
//!     count: usize,
//!     count_fraction: f32,
//!     user: String,
//! }
//!
//! let druid_client = DruidClient::new(vec!["localhost:8082".to_string()]);
//!
//! let timeseries = Timeseries {
//!     data_source: DataSource::table("wikipedia"),
//!     limit: Some(10),
//!     descending: false,
//!     granularity: Granularity::All,
//!     filter: Some(Filter::selector("user", "Taffe316")),
//!     aggregations: vec![
//!         Aggregation::count("count"),
//!         Aggregation::StringFirst {
//!             name: "user".into(),
//!             field_name: "user".into(),
//!             max_string_bytes: 1024,
//!         },
//!     ],
//!     post_aggregations: vec![PostAggregation::Arithmetic {
//!         name: "count_fraction".into(),
//!         function: "/".into(),
//!         fields: vec![
//!             PostAggregator::field_access("count_percent", "count"),
//!             PostAggregator::constant("hundred", 100.into()),
//!         ],
//!         ordering: None,
//!     }],
//!     intervals: vec!["-146136543-09-08T08:23:32.096Z/146140482-04-24T15:36:27.903Z".into()],
//!     context: Default::default(),
//! };
//! let result = druid_client.timeseries::<TimeAggr>(&timeseries);
//!
//! ```

extern crate serde_json;

pub mod client;
pub mod connection;
pub mod query;
pub mod serialization;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
