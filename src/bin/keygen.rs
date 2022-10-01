use ed25519_zebra::{SigningKey, VerificationKey};
use rand::thread_rng;

fn main() -> anyhow::Result<()> {
    let sk = SigningKey::new(thread_rng());
    let sk_bytes: [u8; 32] = sk.into();
    let sk_str = base64_url::encode(&sk_bytes);
    println!("key = {:?}", sk_str);
    let sk_bytes = base64_url::decode(&sk_str)?;
    let decoded_sk = SigningKey::try_from(&sk_bytes[..])?;
    let message = b"hello world";
    let signature = sk.sign(message);
    let vk = VerificationKey::from(&decoded_sk);
    assert!(vk.verify(&signature, message).is_ok());
    Ok(())
}
