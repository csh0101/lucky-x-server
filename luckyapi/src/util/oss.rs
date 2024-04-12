use aliyun_oss_rust_sdk::oss::OSS;
use env_file_reader::read_file;
use std::sync::Arc;
use tokio::sync::OnceCell;

static OSS_INSTANCE: OnceCell<Arc<OSS>> = OnceCell::const_new();

pub async fn get_oss_instance() -> Arc<OSS> {
    OSS_INSTANCE
        .get_or_init(|| async {
            let env_variables = read_file(".env").unwrap();
            for (k, v) in env_variables.iter() {
                std::env::set_var(k, v)
            }
            Arc::new(OSS::from_env())
        })
        .await
        .clone()
}

mod test {
    use super::get_oss_instance;
    use crate::build;
    use aliyun_oss_rust_sdk::request::RequestBuilder;
    use env_file_reader::read_file;

    #[tokio::test]
    async fn test_oss_upload() {
        let env_variables =
            read_file("/home/csh0101/lab/lucky-x-server/.env").unwrap();
        for (k, v) in env_variables.iter() {
            std::env::set_var(k, v)
        }

        // todo add set env var for this test

        let oss = get_oss_instance().await;

        let builder = RequestBuilder::new().with_expire(60);
        oss.put_object_from_file(
            "/luckytest/ziptestfile.zip",
            "/home/csh0101/lab/lucky-x-server/tmp/zipfile/ziptestfile.zip",
            builder,
        )
        .await
        .unwrap();
    }
}
