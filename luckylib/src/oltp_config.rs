use std::time::Duration;

use once_cell::sync::Lazy;
use opentelemetry::global::ObjectSafeTracerProvider;
use opentelemetry::trace::get_active_span;
use opentelemetry::trace::{Span, Tracer, TracerProvider as _};
use opentelemetry::{global, metrics::MeterProvider as _, Key, KeyValue};
use opentelemetry_otlp::{ExportConfig, Protocol, WithExportConfig};
use opentelemetry_prometheus::PrometheusExporter;
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_sdk::{
    export,
    metrics::{
        reader::{DefaultAggregationSelector, DefaultTemporalitySelector},
        PeriodicReader, SdkMeterProvider,
    },
    trace::{self, RandomIdGenerator, Sampler},
    Resource,
};
use tracing_subscriber::Registry;
use tracing_subscriber::{filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};
// pub fn setup_metrics() -> PrometheusExporter {
//     let exporter = opentelemetry_prometheus::exporter()
//         .with_registry(registry.clone())
//         .build()
//         .unwrap();

//     let provider = SdkMeterProvider::builder().with_reader(exporter).build();
// }
// #[warn(dead_code)]

// pub static OLTP_TRACER: Lazy<opentelemetry_sdk::trace::Tracer> = Lazy::new(|| oltp_tracing_init());

pub static OLTP_METER: Lazy<opentelemetry_sdk::metrics::SdkMeterProvider> = Lazy::new(|| {
    let provider = oltp_meter_init();
    #[cfg(feature = "test")]
    {
        let provider = oltp_metrics_test();
        return provider;
    }

    return provider;
});

// pub static Coutner  : Lazy<> = Lazy::new(|| OLTP_METER.meter())
fn oltp_tracing_test() -> opentelemetry_sdk::trace::Tracer {
    let exporter = opentelemetry_stdout::SpanExporter::default();
    let provider = TracerProvider::builder()
        .with_simple_exporter(exporter)
        .build();
    // global::set_tracer_provider(provider);

    let tracer = provider.versioned_tracer(
        "opentelemetry-otlp",
        Some(env!("CARGO_PKG_VERSION")),
        Some("https://opentelemetry.io/schemas/1.21.0"),
        None,
    );
    tracer
    // return global::tracer_provider().tracer("test_app");
}

fn oltp_metrics_test() -> opentelemetry_sdk::metrics::SdkMeterProvider {
    let exporter = opentelemetry_stdout::MetricsExporter::default();
    let reader = PeriodicReader::builder(exporter, opentelemetry_sdk::runtime::Tokio).build();
    SdkMeterProvider::builder().with_reader(reader).build()
}

pub fn oltp_tracing_init() -> opentelemetry_sdk::trace::Tracer {
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317".to_string())
                .with_timeout(Duration::from_secs(3)),
        )
        .with_trace_config(
            trace::config()
                .with_sampler(Sampler::AlwaysOn)
                .with_id_generator(RandomIdGenerator::default())
                .with_max_events_per_span(64)
                .with_max_attributes_per_event(16)
                .with_max_attributes_per_link(16)
                .with_resource(Resource::new(vec![KeyValue::new(
                    "service.name",
                    "example",
                )])),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .unwrap()
}

pub fn oltp_meter_init() -> opentelemetry_sdk::metrics::SdkMeterProvider {
    let export_config = ExportConfig {
        endpoint: "http://localhost:4317".to_string(),
        protocol: Protocol::Grpc,
        timeout: Duration::from_secs(3),
    };
    opentelemetry_otlp::new_pipeline()
        .metrics(opentelemetry_sdk::runtime::Tokio)
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_export_config(export_config),
        )
        .with_resource(Resource::new(vec![KeyValue::new(
            "service.name",
            "example",
        )]))
        .with_period(Duration::from_secs(3))
        .with_timeout(Duration::from_secs(10))
        .with_aggregation_selector(DefaultAggregationSelector::new())
        .with_temporality_selector(DefaultTemporalitySelector::new())
        .build()
        .unwrap()
}

pub fn setup_tracing() -> anyhow::Result<()> {
    #[cfg(feature = "log-debug")]
    std::env::set_var("RUST_LOG", "debug");

    #[cfg(feature = "log-trace")]
    std::env::set_var("RUST_LOG", "trace");

    // 定义传播者
    global::set_text_map_propagator(opentelemetry_jaeger_propagator::Propagator::new());

    // opentelemetry_otlp::new_exporter().tonic()
    // 定义opentelemetry的tracer
    // 定义opentelemetry的layer

    #[cfg(feature = "test")]
    let tracer = oltp_tracing_test();

    #[cfg(not(feature = "test"))]
    let tracer = oltp_tracing_init();

    fn my_other_function() {
        // call methods on the current span from
        get_active_span(|span| {
            span.add_event(
                "An event!".to_string(),
                vec![KeyValue::new("happened", true)],
            );
        })
    }

    let opentelemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(opentelemetry)
        .with(
            fmt::layer()
                .with_line_number(true)
                .with_thread_ids(true)
                .with_target(false),
        )
        .with(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .init();

    // let req_counter = meter.u64_observable_counter("x").init();

    // req_counter.add(1, &[KeyValue::new("key", "value")]);

    // 开启 tokio span htop的能力

    #[cfg(feature = "debug")]
    console_subscriber::init();
    Ok(())
}

mod test {

    use super::oltp_tracing_test;
    use super::OLTP_METER;
    use opentelemetry::{global, metrics::MeterProvider, Key, KeyValue};
    use opentelemetry_otlp::{ExportConfig, Protocol, WithExportConfig};

    use once_cell::sync::Lazy;
    use opentelemetry::global::ObjectSafeTracerProvider;
    use opentelemetry::trace::{Span, Tracer, TracerProvider as _};
    use opentelemetry_prometheus::PrometheusExporter;
    use opentelemetry_sdk::trace::TracerProvider;
    use opentelemetry_sdk::{
        export,
        metrics::{
            reader::{DefaultAggregationSelector, DefaultTemporalitySelector},
            SdkMeterProvider,
        },
        trace::{self, RandomIdGenerator, Sampler},
        Resource,
    };
    use tracing::info;
    use tracing::info_span;
    use tracing::instrument;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::Registry;
    use tracing_subscriber::{filter::EnvFilter, fmt, util::SubscriberInitExt};

    use opentelemetry::trace::get_active_span;

    #[tokio::test]

    async fn test_counter_add() {
        let meter = OLTP_METER.meter("app");

        let counter = meter
            .u64_counter("request_counter")
            .with_description("http_request")
            .init();
        counter.add(1, &[KeyValue::new("key1", "value1")]);
        counter.add(1, &[KeyValue::new("key1", "value1")]);
    }

    #[tokio::test]
    async fn test_stdout_tracer() {
        let tracer = oltp_tracing_test();

        let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
        let subscriber = Registry::default().with(telemetry);

        tracing::subscriber::set_global_default(subscriber).expect("set global tracer failed");

        let tracer = global::tracer_provider().tracer("my_app");

        fn my_other_function() {
            // call methods on the current span from
            get_active_span(|span| {
                span.add_event(
                    "An event!".to_string(),
                    vec![KeyValue::new("happened", true)],
                );
            })
        }

        tracer.in_span("xyz", |_cx| {
            my_other_function();
        });

        instrument_test("csh0101".to_string());

        // let span_builder = tracer.span_builder("test-builder");
    }

    #[instrument]
    fn instrument_test(xyz: String) {
        info!("test instrument");

        do_something_test("xyz".to_string());
    }

    fn do_something_test(xyz: String) {
        info!("{}", xyz)
    }

    #[tokio::test]
    async fn test_opentelemetry_tracer() {
        super::setup_tracing().expect("unexcepeted error");
        instrument_test("csh0202".to_string());
        println!("start sleep");
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }

    #[tokio::test]
    async fn test_default_config_otlp() {
        fn my_other_function() {
            // call methods on the current span from
            get_active_span(|span| {
                println!("just do it");
                span.add_event(
                    "An event!".to_string(),
                    vec![KeyValue::new("happened", true)],
                );
            })
        }
        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint("https://localhost:4317"),
            )
            .install_batch(opentelemetry_sdk::runtime::Tokio)
            .unwrap();

        tracer.in_span("doing_work_test_e", |cx| {
            my_other_function();
        });
        let mut span = tracer.start("doing_work_test_f");
        my_other_function(); // 确保异步函数被等待
        span.end();
        println!("start sleep");
        // global::shutdown_tracer_provider();
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}
