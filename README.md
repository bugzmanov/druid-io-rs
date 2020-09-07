# Async rust client for Apache Druid 

Fully asynchronous, future-enabled (http://druid.io/)[Apache Druid] client library for rust language.

The library provides staticly typed API for (https://druid.apache.org/docs/latest/querying/querying.html)[native queries] and less strict API for (https://druid.apache.org/docs/latest/querying/sql.html)[sql queries].

## Installation
Library hosted on [crates.io](https://crates.io/crates/druid-io-client/).
```toml
[dependencies]
druid-io-client = "*"
```

## Supported Queries

* TopN
* GroupBy
* Scan
* Search
* TimeBoundary
* SegmentMetadata
* DataSourceMetadata