use crate::common::error::AppError;
use crate::common::response::{build_success_response, AppJson, RespResult};
use crate::models::archive::{Process, ProcessStatus};
use anyhow::{anyhow, Context, Error};
use async_zip::tokio::write::ZipFileWriter;
use async_zip::{Compression, ZipEntryBuilder};
use axum::debug_handler;
use axum::extract::{Json, State};
use clap::builder;
use fs_extra::{dir::*, file};
use md5;
use opentelemetry::KeyValue;
use regex;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io;
use std::ops::Deref;
use std::path::Path;
use std::path::PathBuf;
use std::str;
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc::Sender;
use tokio::task;
use tokio_util::bytes::buf;
use tokio_util::compat::Compat;
use tracing::field::debug;
use tracing::{info, instrument};
use uuid::Uuid;
use walkdir::WalkDir;
use zip::write::FileOptions;
use zip::{self, ZipWriter};

use crate::util::get_oss_instance;
use crate::{build, AppContext};
use aliyun_oss_rust_sdk::request::RequestBuilder;
use std::sync::{Arc, Mutex as _};
use tokio::sync::Mutex;
use tokio::sync::Semaphore;

use crate::db::sqlx_dao::{self, add_archive_process, update_archive_process};

#[debug_handler]
#[instrument(fields(custom_field = "zip archive"), skip(app_context))]
pub async fn zipfile_bundle(
    State(app_context): State<Arc<AppContext>>,
    AppJson(file_bundle): AppJson<FileBundle>,
) -> Result<AppJson<RespResult<ZipFileBundleResult>>, AppError> {
    let mut output_name = {
        if let Some(key) = file_bundle.key {
            output_filename(&file_bundle.filename, &key)
        } else {
            output_filename(&file_bundle.filename, "")
        }
    }?;
    output_name = format!("{}_{}.zip", Uuid::new_v4(), &output_name,);
    let path_clone = file_bundle.path.clone();
    let process_id = add_archive_process(Process::new_pending_process(
        file_bundle.path,
        output_name.clone(),
    ))
    .await?;
    tokio::spawn(async move {
        // #[cfg(feature = "async")]
        // {
        match async_build_zip(
            app_context.clone(),
            path_clone,
            output_name.clone(),
        )
        .await
        {
            Ok((output_name, file_full_path)) => {
                if let Err(e) =
                    upload(app_context.clone(), &output_name, file_full_path)
                        .await
                {
                    tracing::info!("call upload function failed {}", e)
                }
            }
            Err(e) => {
                tracing::error!("async_build_zip error: {}", e);
                match update_archive_process(
                    ProcessStatus::Failed,
                    process_id as i64,
                )
                .await
                {
                    Ok(id) => {
                        tracing::debug!(
                            "update_archive_process success: {}",
                            id
                        )
                    }
                    Err(e) => {
                        tracing::debug!("update_archive_process failed: {}", e)
                    }
                }
            }
        };
        // }
        // output_name = async_build_zip(app_context, file_bundle.clone()).await?;
        // #[cfg(not(feature = "async"))]
        // {
        //     output_name =
        //         std_zipfile_bundle(app_context.clone(), file_bundle.clone())
        //             .await?;
        // }
    });
    let resp =
        ZipFileBundleResult { success: true, process_id, ..Default::default() };

    Ok(AppJson(build_success_response(resp)))
}

#[instrument(fields(custom_field = "upload oss"))]
async fn upload(
    app_context: Arc<AppContext>,
    output_name: &str,
    file_full_path: String,
) -> anyhow::Result<()> {
    let dir = get_dir(&output_name);

    let oss_full_filepath = format!("{}/{}", dir, &output_name);

    let oss = get_oss_instance().await;

    let builder = RequestBuilder::new();
    match oss
        .put_object_from_file(
            oss_full_filepath.clone(),
            file_full_path.clone(),
            builder,
        )
        .await.with_context(|| format!("Failed to put_object_from_file, target_path: {} , local_file_path: {}",oss_full_filepath, file_full_path))
    {
        Ok(()) => {
            info!("upload oss file {}", oss_full_filepath);
            app_context
                .metric_context
                .zip_archive_context
                .oss_upload_file_success
                .add(1, &[]);
        }
        Err(err) => {
            app_context
                .metric_context
                .zip_archive_context
                .oss_upload_file_failed
                .add(1, &[]);
            return Err(err);
        }
    };
    Ok(())
}

async fn std_zipfile_bundle(
    app_context: Arc<AppContext>,
    file_bundle: FileBundle,
) -> Result<String, AppError> {
    let output_name =
        task::spawn_blocking(move || build_zip(file_bundle)).await??;
    Ok(output_name)
}

#[derive(Deserialize, Debug, Clone)]
pub struct FileBundle {
    pub path: String,
    pub deltarget: Option<u8>,
    pub key: Option<String>,
    pub filename: String,
}

#[derive(Serialize, Deserialize, Default)]
pub struct ZipFileBundleResult {
    success: bool,
    process_id: u64,
    oss_url: String,
    status: String,
}

#[instrument(fields(custom_field = "async build zip"), skip(app_context))]
pub async fn async_build_zip(
    app_context: Arc<AppContext>,
    dir_path: String,
    output_name: String,
) -> anyhow::Result<(String, String), AppError> {
    // app_context.metric_context.zip_archive_context.file_archive_content_size.add();

    let archive_file_path =
        Path::new("./tmp").join("zipfile").join(&output_name);

    delete_file_if_exists(&archive_file_path)?;

    let archive_file = tokio::fs::File::create(&archive_file_path).await?;

    let mut zip_writer =
        async_zip::tokio::write::ZipFileWriter::with_tokio(archive_file);

    let mut content_total_size: u64 = 0;

    let filepath: PathBuf = Path::new(&dir_path).into();

    let (tx, mut rx) =
        tokio::sync::mpsc::channel::<(Vec<u8>, ZipEntryBuilder)>(2000);

    let app_context_cloned = Arc::clone(&app_context);

    tokio::spawn(async move {
        match deal_dir_zip_archive(filepath, tx).await {
            Ok(()) => {}
            Err(e) => {
                app_context_cloned
                    .metric_context
                    .zip_archive_context
                    .file_archive_failed
                    .add(1u64, &[]);
            }
        };
    });

    while let Some((buffer, builder)) = rx.recv().await {
        content_total_size += buffer.len() as u64;
        // zip_writer.lock().write_entry_whole(zip_builder, &buffer).await?;
        let _ = zip_writer.write_entry_whole(builder, &buffer).await?;
    }
    zip_writer.close().await?;

    app_context
        .metric_context
        .zip_archive_context
        .file_archive_content_size
        .record(content_total_size, &[]);
    app_context
        .metric_context
        .zip_archive_context
        .file_archive_success
        .add(1u64, &[]);

    Ok((output_name, archive_file_path.to_string_lossy().to_string()))
}

async fn deal_dir_zip_archive(
    filepath: PathBuf,
    tx: tokio::sync::mpsc::Sender<(Vec<u8>, ZipEntryBuilder)>,
) -> anyhow::Result<()> {
    let entries = async_walk_dir(filepath.clone()).await?;

    let input_dir_str = filepath
        .as_os_str()
        .to_str()
        .ok_or(anyhow!("Input path not valid UTF-8."))?
        .to_string();

    let semaphore = Arc::new(Semaphore::new(2000));

    for entry_path_buf in entries {
        let semaphore_clone = semaphore.clone();
        let tx = tx.clone();

        let input_dir_str_clone = input_dir_str.clone();

        tokio::spawn(async move {
            let _permit = semaphore_clone.acquire().await?;
            let entry_path = entry_path_buf.as_path();
            let entry_str = entry_path
                .as_os_str()
                .to_str()
                .ok_or(anyhow!("Directory file path not valid UTF-8."))?;

            if !entry_str.starts_with(&input_dir_str_clone) {
                return Err(anyhow!("Directory file path does not start with base input directory path."));
            }

            let entry_str = &entry_str[&input_dir_str_clone.len() + 1..];
            let mut input_file =
                tokio::fs::File::open(entry_path).await.with_context(|| {
                    format!(
                        "failed to open file, path: {}",
                        entry_path.to_string_lossy()
                    )
                })?;
            let input_file_size = input_file.metadata().await?.len();
            let mut buffer = Vec::with_capacity(input_file_size as usize);
            input_file.read_to_end(&mut buffer).await?;
            tx.send((
                buffer,
                ZipEntryBuilder::new(entry_str.into(), Compression::Stored),
            ))
            .await?;
            Ok::<(), anyhow::Error>(())
        });
    }
    Ok(())
}

async fn write_entry(
    filename: &str,
    input_path: &Path,
    writer: &mut async_zip::tokio::write::ZipFileWriter<&mut tokio::fs::File>,
) -> anyhow::Result<u64> {
    let mut input_file = tokio::fs::File::open(input_path).await?;
    let input_file_size = input_file.metadata().await?.len();

    let mut buffer = Vec::with_capacity(input_file_size as usize);
    input_file.read_to_end(&mut buffer).await?;

    let builder = ZipEntryBuilder::new(filename.into(), Compression::Stored);
    // todo 流式写入的方式貌似不太work,局部文件头无效
    // writer.write_entry_stream(builder).await?;
    writer.write_entry_whole(builder, &buffer).await?;

    Ok(input_file_size)
}

pub fn build_zip(bundle: FileBundle) -> anyhow::Result<String, AppError> {
    // 这里是有可能报错的，但是anyhow捕捉了

    // info!("build_zip");

    let output_name = {
        if let Some(key) = bundle.key {
            output_filename(&bundle.filename, &key)
        } else {
            output_filename(&bundle.filename, "")
        }
    }?;

    let archive_file_path = Path::new("./tmp")
        .join("zipfile")
        .join(format!("{}.zip", &output_name));

    // debug!("the archive_file_path is {:?}", archive_file_path);

    delete_file_if_exists(&archive_file_path)?;

    let archive_file = fs::File::create_new(archive_file_path)?;

    let mut zip_writer = ZipWriter::new(archive_file);

    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .unix_permissions(0o755);

    for filepath in filepaths(&bundle.path) {
        for path in WalkDir::new(Path::new(&filepath)) {
            // 这里又把错误往上丢了, Rust不允许合并行，for循环会移动所有权，这个估计是解糖后的操作
            let path = path?;
            let path = path.path();
            let strip_prefix_name = path.strip_prefix(&filepath)?;
            if path.is_file() {
                zip_writer
                    .start_file(strip_prefix_name.to_string_lossy(), options)?;
                let mut f = File::open(path)?;
                io::copy(&mut f, &mut zip_writer)?;
            } else {
                zip_writer.add_directory(
                    strip_prefix_name.to_string_lossy(),
                    options,
                )?;
            }
        }
    }
    // 完成ZIP文件的创建
    zip_writer.finish().unwrap();

    // 获取内存中的ZIP数据
    Ok(output_name)
}

// todo 这里铁报错
#[instrument]
fn output_filename(filename: &str, key: &str) -> anyhow::Result<String> {
    let name = if filename.is_empty() { key } else { filename };
    let mut output_name = String::new();
    let _ = html_escape::decode_html_entities_to_string(name, &mut output_name);
    return Ok(reg_replace(output_name));
}

#[instrument]
fn reg_replace(output_name: String) -> String {
    let special_chars_pattern = regex::Regex::new(
        r"[`~!@#$^&*()=|{}\[\]:;',\\.<>/?~！@#￥……&*（）——|{}\[\]‘；：”“'。，、？%+\-_]",
    ).expect("regex re is not vaild");
    special_chars_pattern.replace_all(&output_name, "").to_string()
}

fn filepaths(path: &str) -> Vec<String> {
    return gjson::parse(path)
        .array()
        .iter()
        .map(|ele| ele.get("filepath").to_string())
        .collect();
}

#[instrument]
fn delete_file_if_exists(path: &Path) -> anyhow::Result<()> {
    if path.exists() {
        // 如果文件存在，则尝试删除
        fs::remove_file(path)?;
    }
    Ok(())
}

#[instrument]
fn create_dir_all_if_not_exist(path: &Path) -> anyhow::Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

fn get_dir(sm: &str) -> String {
    let num = sm.chars().filter(|c| c.is_digit(10)).collect::<String>();
    let mstr = sm.chars().filter(|c| !c.is_digit(10)).collect::<String>();

    let num_md5 = format!("{:x}", md5::compute(num.as_bytes()));
    let str_md5 = format!("{:x}", md5::compute(mstr.as_bytes()));

    let num1 = num_md5.chars().filter(|c| c.is_digit(10)).collect::<String>();
    let num2 = str_md5.chars().filter(|c| c.is_digit(10)).collect::<String>();

    let mut res_str = format!("{}{}", num1, num2);
    res_str.truncate(10);
    let mut res = res_str.parse::<i64>().unwrap_or(0);

    let mut dir = String::new();
    for _ in 0..3 {
        dir = format!("/{}{}", res % 1000, dir);
        res /= 1000;
    }

    dir
}
//
pub async fn async_walk_dir(dir: PathBuf) -> anyhow::Result<Vec<PathBuf>> {
    let mut dirs = vec![dir];
    let mut files = vec![];

    while !dirs.is_empty() {
        let mut dir_iter = tokio::fs::read_dir(dirs.remove(0)).await?;

        while let Some(entry) = dir_iter.next_entry().await? {
            let entry_path_buf = entry.path();

            if entry_path_buf.is_dir() {
                dirs.push(entry_path_buf);
            } else {
                files.push(entry_path_buf);
            }
        }
    }

    Ok(files)
}

pub fn walk_dir_sync(dir: PathBuf) -> anyhow::Result<Vec<PathBuf>> {
    let mut dirs = vec![dir];
    let mut files = vec![];

    while !dirs.is_empty() {
        // 使用 std::fs::read_dir 替代 tokio::fs::read_dir
        let dir = dirs.remove(0);
        let dir_iter = fs::read_dir(dir)?;

        for entry in dir_iter {
            let entry = entry?;
            let entry_path = entry.path();

            if entry_path.is_dir() {
                dirs.push(entry_path);
            } else {
                files.push(entry_path);
            }
        }
    }

    Ok(files)
}

pub fn walkdir_sync_v2(dir: PathBuf) -> anyhow::Result<Vec<PathBuf>> {
    let mut files = vec![];

    for entry in WalkDir::new(dir) {
        let entry = entry?;
        let path = entry.path().to_path_buf();

        if path.is_file() {
            files.push(path);
        }
    }

    Ok(files)
}

pub async fn archive_procecss_status(
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<AppJson<RespResult<ZipFileBundleResult>>, AppError> {
    let process_id = id.parse::<i64>()?;
    let process = sqlx_dao::query_process_status_by_id(process_id).await?;

    Ok(AppJson(build_success_response(ZipFileBundleResult {
        success: true,
        process_id: process_id as u64,
        oss_url: process.oss_url.unwrap_or_default(),
        status: process.status.into(),
    })))
    // ...
}

mod test {

    use super::get_dir;
    use super::output_filename;
    use super::FileBundle;

    #[test]
    fn test_get_dir() {
        let sm = "example123text";

        let dir = get_dir(sm);

        println!("Directory: {}", dir);

        // assert!()
    }
    #[test]
    fn test_output_name() {
        let input_file_name = "/tmp/pictures/x.y";
        let output_name = output_filename(&input_file_name, "").unwrap();
        println!("output_name : {}", output_name);
        assert_eq!("tmppicturesxy", output_name)
    }

    #[test]
    fn test_serde_file_bundle() {
        let data = r#"{
            "filename" : "zip_test_file",
            "path": "/tmp/pictures"
        }"#;

        let x: FileBundle = serde_json::from_str(&data).unwrap();
        println!("{:?}", x)
    }
}
