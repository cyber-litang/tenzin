use crate::config::{get_config, MailConfig};
use anyhow::{Context, Result};
use async_imap::Session;
use async_native_tls::TlsStream;
use futures::{StreamExt, TryStreamExt};
use tokio::net::TcpStream;

#[derive(Debug)]
pub struct ResetPasswordRequest {
    pub email: String,
    pub student_id: String,
}

#[derive(Debug)]
pub struct RawEmail {
    pub id: u32,
    pub raw: Option<String>,
}

#[derive(Debug)]
pub struct UnreadMails {
    pub parsed: Vec<ResetPasswordRequest>,
    pub raw: Vec<RawEmail>,
}

pub async fn pull_all_unread_from_directory() -> Result<UnreadMails> {
    let MailConfig {
        domain,
        port,
        user,
        password,
        directory,
        ..
    } = &get_config().mail;

    let tls = async_native_tls::TlsConnector::new();

    let client = async_imap::connect((domain as &str, *port), domain, tls)
        .await
        .context("Failed to connect to IMAP server")?;

    let mut imap_session = client
        .login(user, password)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to login to IMAP server: {}", e.0))?;

    imap_session.select(&directory).await?;

    let mut parsed = Vec::new();
    let mut raw = Vec::new();

    let ids = imap_session.search("UNSEEN").await?;

    for id in &ids {
        let mut raw_mail = None;
        match fetch_parse_mail_header(&mut imap_session, *id, &mut raw_mail).await {
            Ok(req) => {
                parsed.push(req);
            }
            Err(e) => {
                raw.push(RawEmail {
                    id: *id,
                    raw: raw_mail,
                });
                tracing::error!("Failed to parse mail header: {}", e);
            }
        }
    }

    let seq = ids
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let _: Vec<_> = imap_session
        .store(&seq, "+FLAGS (\\Seen)")
        .await?
        .try_collect()
        .await?;

    Ok(UnreadMails { parsed, raw })
}

async fn fetch_parse_mail_header(
    imap_session: &mut Session<TlsStream<TcpStream>>,
    id: u32,
    raw: &mut Option<String>,
) -> Result<ResetPasswordRequest> {
    let range = format!("{}:{}", id, id);
    let message = imap_session
        .fetch(&range, "RFC822.HEADER")
        .await?
        .next()
        .await
        .context("Failed to fetch message")??;
    let header = message.header().context("Failed to get header")?;
    let header = String::from_utf8_lossy(header).to_string();
    *raw = Some(header.clone());
    let mut email = None;
    let mut student_id = None;
    for kv in header.split("\r\n") {
        if let Some((key, value)) = kv.split_once(": ") {
            match key {
                "From" => {
                    let email_full = value.split(' ').last().context("Failed to parse email")?;
                    email = Some(
                        email_full
                            .trim_start_matches('<')
                            .trim_end_matches('>')
                            .to_string(),
                    );
                }
                "Subject" => {
                    let student = value.trim().trim_start_matches("ICS@BUPT#").trim_end();
                    student_id = Some(student.to_string());
                }
                _ => {}
            }
        }
    }

    Ok(ResetPasswordRequest {
        email: email.context("Failed to parse email")?,
        student_id: student_id.context("Failed to parse student id")?,
    })
}
