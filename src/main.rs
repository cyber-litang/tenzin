use anyhow::Result;
use serde::Deserialize;
use tenzin::mail::{pull_all_unread_from_directory, send_mail, MailConfig};

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub mail: MailConfig,
}

#[tokio::main]
async fn main() -> Result<()> {
    let config: Config = {
        let s = std::fs::read_to_string("config.toml")?;
        println!("{}", s);
        toml::from_str(&s)?
    };
    let res = pull_all_unread_from_directory(&config.mail).await?;
    println!("Hello, world! {:?}", res);
    let s = b"+UXZO1mWHTvZZOQ-/ICS";
    let s = charset::UTF_7.decode_without_bom_handling(s);
    println!("{:?}", s);
    send_mail(&config.mail, "name1e5s@qq.com", "test", "测试").await?;
    Ok(())
}
