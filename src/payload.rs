use anyhow::Result;
use chrono::Utc;
use ed25519_zebra::VerificationKey;
use serde::{Deserialize, Serialize};

use crate::config::get_config;

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub id: String,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Payload {
    pub request: Vec<u8>,
    pub sign: Vec<u8>,
}

pub fn build_payload(id: &str) -> Result<String> {
    let key = &get_config().sign.key;
    let timestamp = Utc::now().timestamp();
    let request = Request {
        id: id.to_string(),
        timestamp,
    };
    let request = bincode::serialize(&request)?;
    let sign: [u8; 64] = key.sign(&request).into();
    let payload = Payload {
        request,
        sign: sign.to_vec(),
    };
    let payload = bincode::serialize(&payload)?;
    let payload = base64_url::encode(&payload);
    Ok(urlencoding::encode(&payload).to_string())
}

pub fn parse_payload(payload: &str) -> Result<Request> {
    let key = &get_config().sign.key;
    let payload = urlencoding::decode(payload)?;
    let payload = base64_url::decode(payload.as_bytes())?;
    let payload: Payload = bincode::deserialize(&payload)?;
    let sign = ed25519_zebra::Signature::try_from(&payload.sign[..])?;
    let vk = VerificationKey::from(key);
    vk.verify(&sign, &payload.request)?;
    let request: Request = bincode::deserialize(&payload.request)?;
    let timestamp = Utc::now().timestamp();
    if timestamp - request.timestamp > get_config().payload.oudate_secounds as i64 {
        anyhow::bail!("payload is outdated");
    }
    Ok(request)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{set_config, Config, PayloadConfig};

    #[test]
    fn test_build_payload() -> Result<()> {
        let config = Config {
            payload: PayloadConfig {
                oudate_secounds: 60,
            },
            ..Default::default()
        };
        set_config(config);
        let payload = build_payload("test")?;
        let request = parse_payload(&payload)?;
        assert_eq!(request.id, "test");
        Ok(())
    }
}
