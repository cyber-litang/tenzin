use std::{env, path::Path};

use anyhow::Context;
use validator::validate_email;

fn input_email(path: &Path) -> anyhow::Result<()> {
    let mut email = String::new();
    loop {
        println!("Please input your email:");
        std::io::stdin().read_line(&mut email)?;
        email = email.trim().to_string();
        if validate_email(&email) {
            break;
        } else {
            eprintln!("Invalid email: {}", email);
        }
    }
    std::fs::write(path, email)?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let home = env::var("HOME").context("Failed to get $HOME")?;
    let force_change = { env::args().skip(1).any(|s| s.starts_with("-c")) };
    let config_path = Path::new(&home).join(".tenzin");
    if force_change {
        println!("Force change email");
        input_email(&config_path)?;
    } else if config_path.exists() {
        let email = std::fs::read_to_string(&config_path)?.trim().to_string();
        if !validate_email(&email) {
            eprintln!("invalid email: {}", email);
            input_email(&config_path)?;
        }
        println!("Your email is: {}", email);
    } else {
        println!("{} has no config file", home);
        input_email(&config_path)?;
    }
    Ok(())
}
