mod receive;
mod send;
mod worker;

pub use receive::pull_all_unread_from_directory;
pub use send::send_mail;
pub use worker::{send_reset_mail, shutdown_mail_worker, spin_up_mail_worker};
