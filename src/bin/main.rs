use anyhow::Result;
use tenzin::{
    config::{get_config, set_config, Config},
    mail::spin_up_mail_worker,
    server::start_server,
    student::spin_up_student_worker,
};
use tracing::Level;

#[tokio::main]
async fn main() -> Result<()> {
    let config: Config = {
        let s = std::fs::read_to_string("config.toml")?;
        println!("{}", s);
        toml::from_str(&s)?
    };
    set_config(config);

    let file_appender =
        tracing_appender::rolling::hourly(&get_config().log.path, &get_config().log.prefix);
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let _ = tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_writer(non_blocking)
        .with_ansi(false)
        .try_init();

    spin_up_student_worker();
    spin_up_mail_worker();
    start_server().await?;
    Ok(())
}
