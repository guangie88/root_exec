#[macro_use]
extern crate error_chain;
extern crate libc;

#[macro_use]
extern crate log;
extern crate log4rs;
extern crate simple_logger;
extern crate structopt;

#[macro_use]
extern crate structopt_derive;

use std::env;
use std::path::PathBuf;
use std::process::{self, Command};
use structopt::StructOpt;

mod errors {
    error_chain! { }
}

use errors::*;

#[derive(StructOpt, Debug)]
#[structopt(name = "root_exec", about = "Program to run root_exec.sh")]
struct MainConfig {
    #[structopt(short = "l", long = "log-config", help = "Log config file path")]
    log_config_path: Option<String>,
}

fn set_uid_root() -> Result<()> {
    let uid_res = unsafe { libc::setuid(0) };

    if uid_res == 0 {
        Ok(())   
    } else {
        bail!("Need to be root")
    }
}

fn get_script_path() -> Result<PathBuf> {
    let current_exe_path = env::current_exe()
        .chain_err(|| "Unable to get current executable path")?;

    let current_exe_dir = match current_exe_path.parent() {
        Some(current_exe_dir) => current_exe_dir,
        None => bail!("Unable to get current executable directory"),
    };

    let mut exec_path = PathBuf::from(current_exe_dir);
    exec_path.push("root_exec.sh");
    let script_path = exec_path;

    if !script_path.exists() {
        bail!("Unable to find {:?} for command execution", script_path);
    }

    Ok(script_path)
}

fn run() -> Result<()> {
    let config = MainConfig::from_args();

    if let &Some(ref log_config_path) = &config.log_config_path {
        log4rs::init_file(log_config_path, Default::default())
            .chain_err(|| format!("Unable to initialize log4rs logger with the given config file at '{}'", log_config_path))?;
    } else {
        simple_logger::init()
            .chain_err(|| "Unable to initialize default logger")?;
    }

    // check for root access first
    set_uid_root()?;

    info!("Config: {:?}", config);
    
    let exec_path = get_script_path()?;
    
    let child = Command::new(&exec_path)
        .output()
        .chain_err(|| "Unable to run command")?;

    info!("Child: {:?}", child);

    Ok(())
}

fn main() {
    match run() {
        Ok(_) => {
            info!("Program completed!");
            process::exit(0)
        },

        Err(ref e) => {
            error!("Error: {}", e);

            for e in e.iter().skip(1) {
                error!("> Caused by: {}", e);
            }

            process::exit(1);
        },
    }
}
