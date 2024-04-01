use opentelemetry::global;
use tracing_subscriber::{filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

// #[warn(dead_code)]
pub fn setup_tracing() {
    #[cfg(feature = "log-debug")]
    std::env::set_var("RUST_LOG", "debug");

    #[cfg(feature = "log-trace")]
    std::env::set_var("RUST_LOG", "trace");

    // 定义传播者
    global::set_text_map_propagator(opentelemetry_jaeger_propagator::Propagator::new());

    // 定义opentelemetry的tracer
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .install_simple()
        .unwrap();

    // 定义opentelemetry的layer
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

    // 开启 tokio span htop的能力
    #[cfg(feature = "debug")]
    console_subscriber::init()
}
