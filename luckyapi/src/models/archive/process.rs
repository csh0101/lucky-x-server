use std::default;

use serde::Deserialize;
use serde::Serialize;
use sqlx;
use sqlx::types::chrono::DateTime;
use sqlx_macros;
#[derive(Debug, sqlx::FromRow, Default)]
pub struct Process {
    pub id: i64,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    // #[sqlx(rename = "status")]
    pub status: ProcessStatus,
    pub oss_url: Option<String>,
    pub target_dir: String,
    pub zip_filename: String,
}

impl Process {
    pub fn new_pending_process(
        target_dir: String,
        zip_filename: String,
    ) -> Self {
        Process {
            start_time: Some(chrono::Local::now().timestamp()),
            end_time: None,
            status: ProcessStatus::Pending,
            oss_url: None,
            target_dir: target_dir,
            zip_filename: zip_filename,
            ..Default::default()
        }
    }
}

#[derive(Debug, sqlx::Decode, sqlx::Encode)]
pub enum ProcessStatus {
    Pending,
    Finished,
    Failed,
}

impl Default for ProcessStatus {
    fn default() -> Self {
        return ProcessStatus::Pending;
    }
}

impl From<std::string::String> for ProcessStatus {
    fn from(s: std::string::String) -> Self {
        if s.eq("pending") {
            ProcessStatus::Pending
        } else if s.eq("finished") {
            ProcessStatus::Finished
        } else {
            ProcessStatus::Failed
        }
    }
}

impl From<ProcessStatus> for std::string::String {
    fn from(status: ProcessStatus) -> Self {
        match status {
            ProcessStatus::Pending => "pending".to_string(),
            ProcessStatus::Finished => "finished".to_string(),
            ProcessStatus::Failed => "failed".to_string(),
        }
    }
}

macro_rules! impl_enum_type {
    ($ty:ty) => {
        impl sqlx::Type<sqlx::MySql> for $ty {
            fn type_info() -> <sqlx::MySql as sqlx::Database>::TypeInfo {
                <str as sqlx::Type<sqlx::MySql>>::type_info()
            }

            fn compatible(
                ty: &<sqlx::MySql as sqlx::Database>::TypeInfo,
            ) -> bool {
                <str as sqlx::Type<sqlx::MySql>>::compatible(ty)
            }
        }
    };
}

impl_enum_type!(ProcessStatus);
