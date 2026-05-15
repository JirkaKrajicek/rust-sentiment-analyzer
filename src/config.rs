pub struct DbConfig {
    url: String,
    max_pool_size: u32,
    run_migrations: bool,
}

impl DbConfig {
    pub fn from_env() -> Self {
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let max_pool_size = std::env::var("MAX_POOL_SIZE")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .expect("MAX_POOL_SIZE must be a valid integer");
        let run_migrations = std::env::var("RUN_MIGRATIONS")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .expect("RUN_MIGRATIONS must be a valid boolean");
        Self {
            url,
            max_pool_size,
            run_migrations,
        }
    }

    pub fn url(&self) -> String {
        self.url.clone()
    }

    pub fn max_pool_size(&self) -> usize {
        self.max_pool_size as usize
    }

    pub fn run_migrations(&self) -> bool {
        self.run_migrations
    }
}