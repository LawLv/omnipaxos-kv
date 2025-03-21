use omnipaxos_kv::common::kv::KVCommand;
// use std::collections::HashMap;
use sqlx::postgres::PgPoolOptions;
use std::env;
use omnipaxos_kv::common::kv::ConsistencyLevel;
use std::time::Duration;

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
        // // 只有编号为 "1" 的服务器负责建表
        // if let Ok(server_id) = env::var("SERVER_ID") {
        //     if server_id == "1" {
        //         // 执行建表操作
        //         sqlx::query(
        //             "CREATE TABLE IF NOT EXISTS kv_table (
        //                 key TEXT PRIMARY KEY,
        //                 value TEXT
        //             );"
        //         )
        //         .execute(&pool)
        //         .await
        //         .expect("Failed to create kv_table");
        //         log::info!("服务器编号为 1, 已创建 kv_table");
        //     } else {
        //         log::info!("服务器编号不是 1, 不执行建表操作");
        //     }
        // } else {
        //     log::warn!("没有设置 SERVER_ID 环境变量，默认不执行建表操作");
        // }
        // Self { pool }
    }
    pub fn get_pool(&self) -> &sqlx::PgPool {
        &self.pool
    }

    // 修改 handle_command 为 async 并支持 SQL 命令
    pub async fn handle_command(&self, command: KVCommand) -> Option<Option<String>> {
        match command {
            // 对于旧的 Put/Delete/Get，如果还要支持，可以调用内部转换为 SQL 语句
            KVCommand::SQL(query, consistency) => {
                if query.trim().to_uppercase().starts_with("SELECT") {
                    // 执行查询，返回查询到的第一个值（示例中只处理一行一列）
                    match consistency {
                        ConsistencyLevel::Local => {
                            // 本地读取：直接执行查询
                            let result = sqlx::query_scalar::<_, String>(&query)
                                .fetch_one(&self.pool)
                                .await
                                .ok();
                            Some(result)
                        },
                        ConsistencyLevel::Leader => {
                            // Leader读取：假设当前节点是领导者（检查在update_database_and_respond）
                            log::debug!("Executing leader read");
                            let result = sqlx::query_scalar::<_, String>(&query)
                                .fetch_one(&self.pool)
                                .await
                                .ok();
                            Some(result)
                        },
                        ConsistencyLevel::Linearizable => {
                            // 如果需要实现线性化读取，可以在这里等待所有挂起写入应用完毕
                            log::debug!("Executing linearizable read (waiting for log to catch up)");
                            // 例如简单等待一段时间（实际实现可能需要更精细的同步机制）
                            tokio::time::sleep(Duration::from_millis(50)).await;
                            let result = sqlx::query_scalar::<_, String>(&query)
                                .fetch_one(&self.pool)
                                .await
                                .ok();
                            Some(result)
                        },
                    }
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
