use aliyun_oss_rust_sdk::oss::OSS;
use std::sync::Arc;
use tokio::sync::OnceCell;

static OSS_INSTANCE: OnceCell<Arc<OSS>> = OnceCell::const_new();

pub async fn get_oss_instance() -> Arc<OSS> {
    OSS_INSTANCE
        .get_or_init(|| async { Arc::new(OSS::from_env()) })
        .await
        .clone()
}
