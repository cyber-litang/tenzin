use std::sync::Once;

use crate::{
    config::get_config, mail::send_mail, payload::build_payload, student::check_student_email,
};
use anyhow::Result;
use chrono::{Duration, Local};
use once_cell::sync::OnceCell;
use tokio::{
    select,
    sync::mpsc::{channel, Sender},
};
use tracing::{debug, error, info};

use super::pull_all_unread_from_directory;

static WORKER_TX: OnceCell<Sender<()>> = OnceCell::new();

pub fn spin_up_mail_worker() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        tokio::spawn(mail_worker());
    })
}

pub async fn shutdown_mail_worker() {
    if let Some(tx) = WORKER_TX.get() {
        tx.send(())
            .await
            .expect("Failed to send shutdown signal to worker");
    }
}

async fn mail_worker() {
    let (tx, mut rx) = channel(1);
    WORKER_TX.set(tx).expect("worker already initialized");
    let send_duration = get_config().mail.send_duration;
    let check_duration = get_config().mail.check_duration;
    select! {
        _ = rx.recv() => {
            info!("rx: mail worker stopped");
        },
        _ = async {
            loop {
                info!("mail_worker running");
                if let Err(e) = process_mails(send_duration).await {
                    error!("Failed to process_mails: {}", e);
                }
                tokio::time::sleep(std::time::Duration::from_secs(check_duration)).await;
            }
        } => {
            error!("worker stopped");
        }
    }
}

async fn process_mails(send_duration: u64) -> Result<()> {
    let mails = pull_all_unread_from_directory().await?;
    debug!(mails=?mails);
    for req in mails.parsed {
        if !check_student_email(&req.student_id, &req.email) {
            error!("invalid student email: {:?}", req);
            continue;
        }
        if let Err(e) = send_reset_mail(&req.email, &req.student_id).await {
            error!("Failed to send mail: {}", e);
        }
        tokio::time::sleep(std::time::Duration::from_secs(send_duration)).await;
    }
    Ok(())
}

pub async fn send_reset_mail(mail: &str, id: &str) -> Result<()> {
    info!("send reset mail");
    let link = {
        let domain = &get_config().server.domain;
        let port = get_config().server.port;
        let payload = build_payload(id)?;
        format!("http://{}:{}/reset/{}", domain, port, payload)
    };
    let ddl = {
        let ddl = Local::now() + Duration::seconds(get_config().payload.oudate_secounds as _);
        ddl.to_rfc3339()
    };
    let text = format!(
        "请点击以下链接重置密码，链接在 {} 前有效 \n\n {}",
        ddl, link
    );
    send_mail(mail, "重置密码", &text).await?;
    Ok(())
}
