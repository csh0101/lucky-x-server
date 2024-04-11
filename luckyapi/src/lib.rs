// 这种写法已经不在需要了
// #[macro_use]
// extern crate shadow_rs;
use shadow_rs::shadow;
shadow!(build);
pub mod common;
pub mod db;
pub mod error;
pub mod handlers;
pub mod models;

pub mod util;
pub use handlers::health_handler::health_check_handler;
pub use handlers::zip_handler::async_build_zip;
pub use handlers::zip_handler::zipfile_bundle;
use opentelemetry::{
    global,
    metrics::{Counter, Histogram, MeterProvider, Unit},
    Key, KeyValue,
};
pub use util::parallel_copy;

#[derive(Debug)]
pub struct AppContext {
    metric_context: MetricContext,
}

impl AppContext {
    pub fn init() -> Self {
        AppContext { metric_context: MetricContext::init() }
    }
}

/*
 * file_copy represent the copy from nas directory to local file system (LFS)
 * file_archive represent the archive file process
 * oss_upload_file represent the action that upload file to oss.
 */

#[derive(Debug)]
struct MetricContext {
    zip_archive_context: ZipArchiveMetricContext,
}

#[derive(Debug)]
struct ZipArchiveMetricContext {
    file_req_access: Counter<u64>,
    file_copy_success: Counter<u64>,
    file_copy_failed: Counter<u64>,
    file_archive_success: Counter<u64>,
    file_archive_failed: Counter<u64>,
    file_archive_content_size: Histogram<u64>,
    oss_upload_file_success: Counter<u64>,
    oss_upload_file_failed: Counter<u64>,
}

impl MetricContext {
    fn init() -> Self {
        MetricContext {
            zip_archive_context: registry_zip_achrive_metrics().unwrap(),
        }
    }
}

fn registry_zip_achrive_metrics() -> anyhow::Result<ZipArchiveMetricContext> {
    let meter = luckylib::OLTP_METER.meter("luckyapi");

    let file_req_access = meter
        .u64_counter("file_req_access")
        .with_description("represent the file deal req")
        .with_unit(Unit::new("times"))
        .init();

    let file_copy_success = meter
        .u64_counter("file_copy_success")
        .with_description("file_copy_success")
        .with_unit(Unit::new("times"))
        .init();

    let file_copy_failed = meter
        .u64_counter("file_copy_failed")
        .with_description("represent the copy failed count")
        .with_unit(Unit::new("times"))
        .init();

    let file_archive_success = meter
        .u64_counter("file_archive_success")
        .with_description("file archive success count")
        .with_unit(Unit::new("times"))
        .init();

    let file_archive_failed = meter
        .u64_counter("file_archive_failed")
        .with_description("file archive failed count")
        .with_unit(Unit::new("times"))
        .init();

    let file_archive_content_size = meter
        .u64_histogram("file_archive_content_size")
        .with_description("file content size")
        .with_unit(Unit::new("Gb"))
        .init();

    let oss_upload_file_success = meter
        .u64_counter("oss_upload_file_success")
        .with_description("oss upload file success")
        .with_unit(Unit::new("times"))
        .init();

    let oss_upload_file_failed = meter
        .u64_counter("oss_upload_file_failed")
        .with_description("oss upload file failed")
        .with_unit(Unit::new("times"))
        .init();

    Ok(ZipArchiveMetricContext {
        file_req_access,
        file_copy_success,
        file_copy_failed,
        file_archive_success,
        file_archive_failed,
        file_archive_content_size,
        oss_upload_file_success,
        oss_upload_file_failed,
    })
}
