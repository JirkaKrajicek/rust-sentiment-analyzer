use std::{net::SocketAddr, path::PathBuf, time::Duration};

pub struct AppConfig {
    pub database: DbConfig,
    pub inference: InferenceConfig,
    pub model: ModelConfig,
    pub server: ServerConfig,
    pub retention: RetentionConfig,
}

pub struct DbConfig {
    url: String,
    max_pool_size: usize,
    run_migrations: bool,
}

#[derive(Clone, Copy)]
pub struct InferenceConfig {
    pub predict_max_body_bytes: usize,
    pub max_tokens: usize,
    pub queue_timeout: Duration,
    pub execution_timeout: Duration,
    pub document_max_body_bytes: usize,
    pub document_max_characters: usize,
    pub document_extraction_concurrency: usize,
    pub document_extraction_queue_timeout: Duration,
    pub document_extraction_timeout: Duration,
}

pub struct ModelConfig {
    pub model_path: PathBuf,
    pub tokenizer_path: PathBuf,
}

pub struct ServerConfig {
    pub bind_address: SocketAddr,
}

pub struct RetentionConfig {
    pub result_retention_days: u32,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database: DbConfig::from_env()?,
            inference: InferenceConfig::from_env()?,
            model: ModelConfig::from_env()?,
            server: ServerConfig::from_env()?,
            retention: RetentionConfig::from_env()?,
        })
    }
}

impl DbConfig {
    fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            url: required_env("DATABASE_URL")?,
            max_pool_size: positive_usize_env("MAX_POOL_SIZE", 10)?,
            run_migrations: boolean_env("RUN_MIGRATIONS", true)?,
        })
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn max_pool_size(&self) -> usize {
        self.max_pool_size
    }

    pub fn run_migrations(&self) -> bool {
        self.run_migrations
    }
}

impl InferenceConfig {
    fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            predict_max_body_bytes: positive_usize_env("PREDICT_MAX_BODY_BYTES", 64 * 1024)?,
            max_tokens: positive_usize_env("MODEL_MAX_TOKENS", 512)?,
            queue_timeout: Duration::from_millis(positive_env(
                "INFERENCE_QUEUE_TIMEOUT_MS",
                1_000,
            )?),
            execution_timeout: Duration::from_millis(positive_env("INFERENCE_TIMEOUT_MS", 10_000)?),
            document_max_body_bytes: positive_usize_env(
                "DOCUMENT_MAX_BODY_BYTES",
                10 * 1024 * 1024,
            )?,
            document_max_characters: positive_usize_env("DOCUMENT_MAX_CHARACTERS", 1_000_000)?,
            document_extraction_concurrency: positive_usize_env(
                "DOCUMENT_EXTRACTION_CONCURRENCY",
                1,
            )?,
            document_extraction_queue_timeout: Duration::from_millis(positive_env(
                "DOCUMENT_EXTRACTION_QUEUE_TIMEOUT_MS",
                1_000,
            )?),
            document_extraction_timeout: Duration::from_millis(positive_env(
                "DOCUMENT_EXTRACTION_TIMEOUT_MS",
                10_000,
            )?),
        })
    }
}

impl ModelConfig {
    fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            model_path: PathBuf::from(non_empty_env_or("MODEL_PATH", "models/model.onnx")?),
            tokenizer_path: PathBuf::from(non_empty_env_or(
                "TOKENIZER_PATH",
                "models/tokenizer.json",
            )?),
        })
    }
}

impl ServerConfig {
    fn from_env() -> anyhow::Result<Self> {
        let value = non_empty_env_or("SERVER_ADDRESS", "0.0.0.0:3000")?;
        let bind_address = value
            .parse()
            .map_err(|_| anyhow::anyhow!("SERVER_ADDRESS must be a valid host:port address"))?;
        Ok(Self { bind_address })
    }
}

impl RetentionConfig {
    fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            result_retention_days: positive_u32_env("SENTIMENT_RESULT_RETENTION_DAYS", 30)?,
        })
    }
}

fn required_env(name: &str) -> anyhow::Result<String> {
    std::env::var(name).map_err(|_| anyhow::anyhow!("{name} must be set"))
}

fn non_empty_env_or(name: &str, default: &str) -> anyhow::Result<String> {
    let value = std::env::var(name).unwrap_or_else(|_| default.to_string());
    if value.trim().is_empty() {
        anyhow::bail!("{name} must not be empty");
    }
    Ok(value)
}

fn positive_usize_env(name: &str, default: usize) -> anyhow::Result<usize> {
    let value = positive_env(name, default as u64)?;
    usize::try_from(value).map_err(|_| anyhow::anyhow!("{name} is too large"))
}

fn positive_u32_env(name: &str, default: u32) -> anyhow::Result<u32> {
    let value = positive_env(name, u64::from(default))?;
    u32::try_from(value).map_err(|_| anyhow::anyhow!("{name} is too large"))
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

fn boolean_env(name: &str, default: bool) -> anyhow::Result<bool> {
    std::env::var(name)
        .unwrap_or_else(|_| default.to_string())
        .parse()
        .map_err(|_| anyhow::anyhow!("{name} must be true or false"))
}
