use anyhow::Result;
use fs_extra::dir::*;
pub async fn parallel_copy(from_dir: &str, to_dir: &str) -> Result<u64> {
    Ok(fs_extra::dir::copy(&from_dir, &to_dir, &CopyOptions::new())?)
}
