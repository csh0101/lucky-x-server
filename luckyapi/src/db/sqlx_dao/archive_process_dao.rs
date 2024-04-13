use anyhow::{Ok, Result};
use sqlx::MySql;

use super::manager::DbManager;
use crate::models::archive::{Process, ProcessStatus};

pub async fn add_archive_process(process: Process) -> Result<u64> {
    let res = sqlx::query::<MySql>(
        "insert into archive_process (
            start_time,
            end_time,
            status,
            oss_url,
            target_dir,
            zip_filename) 
            values (?,?,?,?,?,?)",
    )
    .bind(process.start_time)
    .bind(process.end_time)
    .bind(process.status)
    .bind(process.oss_url)
    .bind(process.target_dir)
    .bind(process.zip_filename)
    .execute(DbManager::get_instance().pool())
    .await?;

    Ok(res.last_insert_id())
}

pub async fn delete_archive_process(id: i64) -> Result<u64> {
    let res = sqlx::query("delete from archive_process where id = ?")
        .bind(id)
        .execute(DbManager::get_instance().pool())
        .await?;
    Ok(res.last_insert_id())
}

pub async fn update_archive_process(
    status: ProcessStatus,
    id: i64,
) -> Result<u64> {
    let res = sqlx::query_as!(
        Process,
        "update archive_process set status = ? where id = ?",
        status,
        id
    )
    .execute(DbManager::get_instance().pool())
    .await?;
    Ok(res.last_insert_id())
}

pub async fn query_process_status_by_id(id: i64) -> Result<Process> {
    let process = sqlx::query_as!(
        Process,
        "select *  from archive_process where id = ?",
        id
    )
    .fetch_one(DbManager::get_instance().pool())
    .await?;

    Ok(process)
}

mod test {

    use std::env::set_var;

    use sqlx::query;

    use super::*;

    use crate::models::archive::{self, Process, ProcessStatus};

    #[tokio::test]
    async fn test_crud() {
        set_var(
            "DATABASE_URL",
            "mysql://csh0101:123456@localhost:3306/lucky?charset=utf8",
        );
        let add_id =
            add_archive_process(archive::Process::new_pending_process(
                "pictures".to_string(),
                "test.zip".to_string(),
            ))
            .await
            .unwrap();

        println!("{:?}", add_id);

        let process = query_process_status_by_id(add_id as i64).await.unwrap();

        println!(
            "{:?}
        ",
            process
        );

        let update_id =
            update_archive_process(ProcessStatus::Finished, 1).await;

        println!("{:?}", update_id);

        let delete_id = delete_archive_process(add_id as i64).await.unwrap();

        println!("{:?}", delete_id)
    }
}
