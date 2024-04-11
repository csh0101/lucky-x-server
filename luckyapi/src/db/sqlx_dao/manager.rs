use once_cell::sync::Lazy;
use sqlx::{MySql, Pool};
use std::env;

// 定义一个数据库管理器结构体，它包含了一个sqlx的Pool<MySql>类型的连接池
pub struct DbManager {
    pool: Pool<MySql>,
}

impl DbManager {
    // 使用单例模式和工厂模式结合，通过一个静态方法提供对唯一实例的访问
    pub fn get_instance() -> &'static Self {
        static INSTANCE: Lazy<DbManager> = Lazy::new(|| {
            let database_url =
                env::var("DATABASE_URL").expect("DATABASE_URL must be set");
            let pool = sqlx::mysql::MySqlPoolOptions::new()
                .max_connections(5)
                .connect_lazy(&database_url)
                .expect("Failed to create pool.");
            DbManager { pool }
        });
        &INSTANCE
    }

    // 提供一个方法来访问连接池，这样就可以在应用中使用DbManager来进行数据库操作了
    pub fn pool(&self) -> &Pool<MySql> {
        &self.pool
    }
}
