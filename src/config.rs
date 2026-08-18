use std::time::Duration;

pub struct DbConfig {
    url: String,
    max_pool_size: u32,
    run_migrations: bool,
}

#[derive(Clone, Copy)]
pub struct InferenceConfig {
    pub predict_max_body_bytes: usize,
    pub max_tokens: usize,
    pub queue_timeout: Duration,
    pub execution_timeout: Duration,
}

impl InferenceConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            predict_max_body_bytes: positive_usize_env("PREDICT_MAX_BODY_BYTES", 64 * 1024)?,
            max_tokens: positive_usize_env("MODEL_MAX_TOKENS", 512)?,
            queue_timeout: Duration::from_millis(positive_env(
                "INFERENCE_QUEUE_TIMEOUT_MS",
                1_000,
            )?),
            execution_timeout: Duration::from_millis(positive_env("INFERENCE_TIMEOUT_MS", 10_000)?),
        })
    }
}

fn positive_usize_env(name: &str, default: usize) -> anyhow::Result<usize> {
    let value = positive_env(name, default as u64)?;
    usize::try_from(value).map_err(|_| anyhow::anyhow!("{name} is too large"))
}

fn positive_env(name: &str, default: u64) -> anyhow::Result<u64> {
    let value = std::env::var(name)
        .unwrap_or_else(|_| default.to_string())
        .parse::<u64>()
        .map_err(|_| anyhow::anyhow!("{name} must be a positive integer"))?;
    if value == 0 {
        anyhow::bail!("{name} must be a positive integer");
    }
    Ok(value)
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
