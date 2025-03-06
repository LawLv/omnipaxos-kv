use omnipaxos_kv::common::kv::KVCommand;
// use std::collections::HashMap;
use sqlx::postgres::PgPoolOptions;
use std::env;

pub struct Database {
    pool: sqlx::PgPool,
}

impl Database {
    pub async fn new() -> Self {
        // 从环境变量中获取数据库连接字符串
        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL environment variable must be set");
        // 创建连接池
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to Postgres");
        // 执行一次建表操作，假设我们操作的表名为 kv_table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS kv_table (
                key TEXT PRIMARY KEY,
                value TEXT
            );"
        )
        .execute(&pool)
        .await
        .expect("Failed to create kv_table");
        Self { pool }
    }

    // 修改 handle_command 为 async 并支持 SQL 命令
    pub async fn handle_command(&self, command: KVCommand) -> Option<Option<String>> {
        match command {
            // 对于旧的 Put/Delete/Get，如果还要支持，可以调用内部转换为 SQL 语句
            KVCommand::SQL(query, _consistency) => {
                // match consistency {
                //     ConsistencyLevel::Leader => {
                //         // 检查当前节点是否为领导者，如果不是则考虑转发或返回错误（这部分可能需要在 server 层处理）
                //         // 此处示例中直接执行查询
                //     },
                //     ConsistencyLevel::Local => { /* 直接执行查询 */ },
                //     ConsistencyLevel::Linearizable => {
                //         // 这里可以添加等待所有日志应用完毕的逻辑，确保读取的线性化一致性
                //     },
                // }
                // 简单判断：以 SELECT 开头的视为读取，否则为写入
                if query.trim().to_uppercase().starts_with("SELECT") {
                    // 执行查询，返回查询到的第一个值（示例中只处理一行一列）
                    let result = sqlx::query_scalar::<_, String>(&query)
                        .fetch_one(&self.pool)
                        .await
                        .ok();
                    Some(result)
                } else {
                    // 写入操作
                    let _ = sqlx::query(&query)
                        .execute(&self.pool)
                        .await;
                    None
                }
            }
            // 如果有其它类型的命令也可以继续支持
            other => {
                // 你可以将其它命令转换为 SQL 语句，或者直接忽略
                println!("Unsupported command: {:?}", other);
                None
            }
        }
    }
}
