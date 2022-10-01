use anyhow::{bail, Result};
use std::{
    io::Write,
    process::{Command, Stdio},
};
use tracing::debug;

pub fn reset_password_for(id: &str) -> Result<()> {
    debug!("Resetting password for {}", id);
    #[cfg(target_os = "macos")]
    const CMD: &str = "cat";
    #[cfg(not(target_os = "macos"))]
    const CMD: &str = "chpasswd";

    let mut cmd = Command::new(CMD).stdin(Stdio::piped()).spawn()?;
    {
        let mut stdin = cmd.stdin.take().expect("Failed to open stdin");
        // user:password
        writeln!(stdin, "{}:bupt{}", id, id)?;
    }
    let status = cmd.wait()?;
    if !status.success() {
        bail!("Failed to reset password for {}", id);
    }

    Ok(())
}

pub fn set_password_expire_for(id: &str) -> Result<()> {
    debug!("Setting password expire for {}", id);
    #[cfg(target_os = "macos")]
    const CMD: &str = "echo";
    #[cfg(not(target_os = "macos"))]
    const CMD: &str = "passwd";

    let mut cmd = Command::new(CMD);

    #[cfg(target_os = "macos")]
    cmd.arg("passwd");

    let status = cmd.args(["-e", id]).status()?;
    if !status.success() {
        bail!("Failed to set password expire for {}", id);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::Level;

    #[test]
    fn test_reset_password() {
        let _ = tracing_subscriber::fmt()
            .with_max_level(Level::DEBUG)
            .try_init();
        reset_password_for("201800000000").unwrap();
    }

    #[test]
    fn test_set_password_expire() {
        let _ = tracing_subscriber::fmt()
            .with_max_level(Level::DEBUG)
            .try_init();
        set_password_expire_for("201800000000").unwrap();
    }
}
