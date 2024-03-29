use crate::db::postgres::PostgresDB;
use sea_orm::{Database, DatabaseConnection, DbErr};

pub mod postgres;

pub struct DB {
    pub postgres_db: PostgresDB,
}

impl DB {
    #[tokio::main]
    pub async fn init(postgres_uri: &str, postgres_name: &str) -> Option<DB> {
        let postgres = DB::init_postgres(postgres_uri, postgres_name).await;

        if let Err(ref e) = postgres {
            println!("Error when connecting to Postgres: {:?}", e);
            return None;
        }

        Some(DB {
            postgres_db: PostgresDB {
                db: postgres.unwrap(),
            },
        })
    }

    async fn init_postgres(uri: &str, name: &str) -> Result<DatabaseConnection, DbErr> {
        // let statement = format!("SELECT \'CREATE DATABASE {}\' WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = \'{}\')", name, name);
        // println!("{}", statement);
        // let db = Database::connect(uri).await?;
        // db.execute(Statement::from_string(db.get_database_backend(), statement))
        //     .await?;

        let uri = format!("{}/{}", uri, name);
        println!("Trying to connect to {}", uri);
        let db_con = Database::connect(&uri).await?;

        Ok(db_con)
    }
}
