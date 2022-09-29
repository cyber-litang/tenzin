mod receive;
mod send;

pub use receive::pull_all_unread_from_directory;
pub use send::send_mail;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct MailConfig {
    pub domain: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub directory: String,
}
