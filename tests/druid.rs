extern crate druid_io;
extern crate tokio;

use druid_io::{
    client::DruidClient,
    query::search::Search,
    query::timeseries::Timeseries,
    query::top_n::TopN,
    query::{
        definitions::Aggregation,
        definitions::{Dimension, Filter, Granularity, Ordering, OutputType, SortingOrder},
        group_by::{
            GroupBy, GroupByBuilder, HavingSpec, LimitSpec, OrderByColumnSpec, PostAggregation,
            PostAggregator,
        },
        scan::{ResultFormat, Scan},
        search::SearchQuerySpec,
        segment_metadata::{AnalysisType, SegmentMetadata, ToInclude},
        time_boundary::{TimeBoundType, TimeBoundary},
        DataSource, JoinType,
    },
};
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
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
    let druid_client = DruidClient::new(vec!["localhost:8082".to_string()]);
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
    // count: usize,
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
                            "-146136543-09-08T08:23:32.096Z/146140482-04-24T15:36:27.903Z".into(),
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

    let druid_client = DruidClient::new(vec!["localhost:8082".to_string()]);
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
    let druid_client = DruidClient::new(vec!["localhost:8082".to_string()]);
    let result = tokio_test::block_on(druid_client.group_by::<WikiPage>(&group_by));
    println!("{:?}", result.unwrap());
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TimeAggr {
    count: usize,
    count_ololo: f32,
    user: String,
}
#[test]
fn test_timeseries() {
    let mut context = HashMap::new();
    context.insert("grandTotal".to_string(), "true".to_string());

    let timeseries = Timeseries {
        data_source: DataSource::table("wikipedia"),
        limit: Some(10),
        descending: false,
        granularity: Granularity::All,
        filter: None, //Some(Filter::selector("user", "Taffe316")),
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
        context: context,
    };
    let druid_client = DruidClient::new(vec!["localhost:8082".to_string()]);
    let result = tokio_test::block_on(druid_client.timeseries::<TimeAggr>(&timeseries));
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
        // .add_context("resultAsArray", "true")
        .build();
    let druid_client = DruidClient::new(vec!["localhost:8082".to_string()]);
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
    let druid_client = DruidClient::new(vec!["localhost:8082".to_string()]);
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
    let druid_client = DruidClient::new(vec!["localhost:8082".to_string()]);
    let result = tokio_test::block_on(druid_client.time_boundary::<WikiPage>(&top_n));
    println!("{:?}", result.unwrap());
}
#[test]
fn test_data_source_metadata() {
    let druid_client = DruidClient::new(vec!["localhost:8082".to_string()]);
    let result =
        tokio_test::block_on(druid_client.datasource_metadata(DataSource::table("wikipedia")));
    println!("{:?}", result.unwrap());
}
#[test]
fn test_segment_metadata() {
    let segment_query = SegmentMetadata {
        data_source: DataSource::table("countries"),
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

    let druid_client = DruidClient::new(vec!["localhost:8082".to_string()]);
    let result = tokio_test::block_on(druid_client.segment_metadata(&segment_query));
    println!("{:?}", result.unwrap());
}
