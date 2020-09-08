# Async rust client for Apache Druid 

<div style="text-align:center"><img src="https://user-images.githubusercontent.com/502482/92421491-c26ab800-f146-11ea-80af-0da8ce4a457d.png" width="10%"/></div>

Fully asynchronous, future-enabled [Apache Druid](http://druid.io/) client library for rust programming language.

The library provides staticly typed API for [Native Queries](https://druid.apache.org/docs/latest/querying/querying.html) 

## Installation
The library is hosted on [crates.io](https://crates.io/crates/druid-io/).

```toml
[dependencies]
druid-io = "*"
```

## Supported Native Queries

* Timeseries
* TopN
* GroupBy
* Scan
* Search
* TimeBoundary
* SegmentMetadata
* DataSourceMetadata

## Usage

### Client

Connect to a druid cluster throughly staticly provided list of brokers:

```rust

let druid_client = DruidClient::new(vec!["localhost:8082".to_string()]);
```

Connector to Druid cluster through Zookeeper - supports autodiscovery of new brokers and load balancing:

```rust

TODO:
```

### Querying

#### Timeseries

See [Timeseries query documentation](https://druid.apache.org/docs/latest/querying/timeseriesquery.html)

```rust
#[derive(Serialize, Deserialize, Debug)]
pub struct TimeAggr {
    count: usize,
    count_fraction: f32,
    user: String,
}

let timeseries = Timeseries {
    data_source: DataSource::table("wikipedia"),
    limit: Some(10),
    descending: false,
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
        name: "count_fraction".into(),
        function: "/".into(),
        fields: vec![
            PostAggregator::field_access("count_percent", "count"),
            PostAggregator::constant("hundred", 100.into()),
        ],
        ordering: None,
    }],
    intervals: vec!["-146136543-09-08T08:23:32.096Z/146140482-04-24T15:36:27.903Z".into()],
    context: context,
};
let result = tokio_test::block_on(druid_client.timeseries::<TimeAggr>(&timeseries));

```

#### TopN
See [Apache Druid TopN query documentation](https://druid.apache.org/docs/latest/querying/topnquery.html)

```rust
#[derive(Serialize, Deserialize, Debug)]
struct WikiPage {
    page: String,
    user: Option<String>,
    count: usize,
}

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
    context: Default::default(),
};
let druid_client = DruidClient::new(vec!["localhost:8082".to_string()]);
let result = tokio_test::block_on(druid_client.top_n::<WikiPage>(&top_n));

```

#### GroupBy
See [Apache Druid GroupBy query documentation](https://druid.apache.org/docs/latest/querying/groupbyquery.html)

```rust

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
        name: "count_fraction".into(),
        function: "/".into(),
        fields: vec![
            PostAggregator::field_access("count_percent", "count"),
            PostAggregator::constant("hundred", 100.into()),
        ],
        ordering: None,
    }],
    having: Some(HavingSpec::greater_than("count_fraction", 0.01.into())),
    intervals: vec!["-146136543-09-08T08:23:32.096Z/146140482-04-24T15:36:27.903Z".into()],
    subtotal_spec: Default::default(),
    context: Default::default(),
};
let result = tokio_test::block_on(druid_client.group_by::<WikiPage>(&group_by));

```

#### Scan (with inner join)
See [Apache Druid TimeBoundary query documentation](https://druid.apache.org/docs/latest/querying/scan-query.html)

Let's try something more complex: inner join

```rust
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
}
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
    context: Default::default(),
};

let result = tokio_test::block_on(druid_client.scan::<ScanEvent>(&scan));

```
