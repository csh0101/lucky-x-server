#[derive(sqlx::FromRow)]
pub struct Process {
    id: u64,
    create_time: u64,
    end_time: u64,
    status: ProcessStatus,
    oss_url: String,
    target_dir: String,
    zip_filename: String,
}

enum ProcessStatus {
    Pending,
    Finished,
    Failed,
}
