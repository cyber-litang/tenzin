use ed25519_zebra::SigningKey;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Deserializer};

static CONFIG: OnceCell<Config> = OnceCell::new();

pub fn get_config() -> &'static Config {
    CONFIG.get().expect("config not initialized")
}

pub fn set_config(config: Config) {
    CONFIG.set(config).expect("config already initialized");
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Config {
    pub mail: MailConfig,
    pub sign: SignConfig,
    pub payload: PayloadConfig,
    pub student: StudentConfig,
    pub server: ServerConfig,
    pub log: LogConfig,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct MailConfig {
    pub domain: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub directory: String,
    pub check_duration: u64,
    pub send_duration: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SignConfig {
    #[serde(deserialize_with = "deserialize_signing_key")]
    pub key: SigningKey,
}

impl Default for SignConfig {
    fn default() -> Self {
        Self {
            key: SigningKey::new(rand::thread_rng()),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct PayloadConfig {
    pub oudate_secounds: u64,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct StudentConfig {
    pub home_prefix: String,
    pub walk_duration: u64,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ServerConfig {
    pub domain: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct LogConfig {
    pub path: String,
    pub prefix: String,
}

fn deserialize_signing_key<'de, D>(deserializer: D) -> Result<SigningKey, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let s = base64_url::decode(&s).map_err(serde::de::Error::custom)?;
    SigningKey::try_from(&s[..]).map_err(serde::de::Error::custom)
}
