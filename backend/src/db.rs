use anyhow::Result;
use sqlx::postgres::PgPool;

pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn connect(url: &str) -> Result<Self> {
        let pool = PgPool::connect(url).await?;
        sqlx::query("SELECT 1").execute(&pool).await?;
        Ok(Self { pool })
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn migrate(&self) -> Result<()> {
        // Run migrations
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }
}
