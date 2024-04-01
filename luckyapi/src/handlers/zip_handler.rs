use axum::{debug_handler, Json};
use core::arch;
use md5;
use regex;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::str;
use std::{
    env,
    fs::{self, File},
};
use tracing::{debug, info};
use zip::write::FileOptions;

use serde::{Deserialize, Serialize};
use walkdir::WalkDir;
use zip::{self, ZipWriter};

use crate::common::error::AppError;
use crate::common::response::{build_success_response, AppJson, RespResult};

#[debug_handler]
pub async fn zipfile_bundle(
    AppJson(params): AppJson<FileBundle>,
) -> Result<AppJson<RespResult<ZipFileBundleResult>>, AppError> {
    let output_name = build_zip(params)?;
    Ok(AppJson(build_success_response(ZipFileBundleResult {
        oss_url: output_name,
    })))
}
#[derive(Deserialize, Debug)]
pub struct FileBundle {
    pub path: String,
    pub deltarget: Option<u8>,
    pub key: Option<String>,
    pub filename: String,
}

#[derive(Serialize, Deserialize)]
pub struct ZipFileBundleResult {
    oss_url: String,
}

pub fn build_zip(bundle: FileBundle) -> anyhow::Result<String, AppError> {
    // 这里是有可能报错的，但是anyhow捕捉了

    // info!("build_zip");

    let output_name = {
        if let Some(key) = bundle.key {
            output_name(&bundle.filename, &key)
        } else {
            output_name(&bundle.filename, "")
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
fn output_name(filename: &str, key: &str) -> anyhow::Result<String> {
    let name = if filename.is_empty() { key } else { filename };
    let mut output_name = String::new();
    let _ = html_escape::decode_html_entities_to_string(name, &mut output_name);
    return Ok(reg_replace(output_name));
}

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
        .map(|ele| ele.to_string())
        .collect();
}

fn delete_file_if_exists(path: &Path) -> anyhow::Result<()> {
    if path.exists() {
        // 如果文件存在，则尝试删除
        fs::remove_file(path)?;
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
pub async fn walk_dir(dir: PathBuf) -> anyhow::Result<Vec<PathBuf>> {
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

mod test {

    use super::get_dir;
    use super::output_name;
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
        let output_name = output_name(&input_file_name, "").unwrap();
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
