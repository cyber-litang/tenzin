use crate::config::get_config;
use anyhow::Result;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::{
    collections::HashMap,
    sync::mpsc::{sync_channel, Receiver, SyncSender, TryRecvError},
};
use tracing::{debug, error, info};
use walkdir::WalkDir;

// id -> email
static STUDENTS: Lazy<RwLock<HashMap<String, String>>> = Lazy::new(|| RwLock::new(HashMap::new()));
static THREAD_TX: Lazy<SyncSender<()>> = Lazy::new(|| {
    let (tx, rx) = sync_channel(1);
    start_student_worker(rx).expect("Failed to start student worker");
    tx
});

pub fn spin_up_student_worker() {
    let sender = THREAD_TX.clone();
    debug!("Spinning up student worker: {:?}", sender);
}

pub fn shutdown_student_worker() {
    debug!("Shutting down student worker");
    THREAD_TX
        .send(())
        .expect("Failed to send shutdown signal to student worker");
}

pub fn check_student_email(id: &str, email: &str) -> bool {
    let students = STUDENTS.read();
    if let Some(student_email) = students.get(id) {
        if student_email == email {
            return true;
        }
    }
    false
}

fn start_student_worker(rx: Receiver<()>) -> Result<()> {
    walk_student_dir()?;
    std::thread::Builder::new()
        .name("t:sdtudent".to_string())
        .spawn(move || loop {
            let duration = std::time::Duration::from_secs(get_config().student.walk_duration);
            std::thread::sleep(duration);
            debug!("start walk student dir");
            if let Err(e) = walk_student_dir() {
                error!("Failed to walk student dir: {}", e);
            }
            match rx.try_recv() {
                Ok(_) => {
                    info!("student worker exit");
                    break;
                }
                Err(TryRecvError::Disconnected) => {
                    error!("channel disconnected, student worker exit");
                    break;
                }
                Err(_) => {}
            }
        })?;
    Ok(())
}

fn walk_student_dir() -> Result<()> {
    let mut students = STUDENTS.write();
    let path = &get_config().student.home_prefix;
    for entry in WalkDir::new(path).min_depth(1).max_depth(1) {
        let entry = entry?;
        let path = entry.file_name();
        let tz_config = entry.path().join(".tenzin");
        if tz_config.exists() {
            let f = || -> anyhow::Result<String> {
                let email = std::fs::read_to_string(&tz_config)?.trim().to_string();
                if !validator::validate_email(&email) {
                    return Err(anyhow::anyhow!(format!("invalid email: {}", email)));
                }
                Ok(email)
            };
            match f() {
                Ok(email) => {
                    students.insert(entry.file_name().to_string_lossy().to_string(), email);
                }
                Err(e) => {
                    error!(
                        "Failed to read email from {}: {}",
                        path.to_string_lossy(),
                        e
                    );
                }
            }
        } else {
            info!("{} has no .tenzin file", path.to_string_lossy());
        }
    }
    debug!(?students);
    Ok(())
}
