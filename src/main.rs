
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{PathBuf, Path};

extern crate anyhow;
use anyhow::{anyhow, Result};
extern crate clap;
use clap::{Arg, App};


mod setups;
mod cli;


fn main() {

    let matches = App::new("moin-dm")
        .version("0.2")
        .author("Simeon Ricking <simeon.ricking@posteo.de>")
        .about("A command line interface for logging in and selecting a user session.")
        .arg(Arg::with_name("config-dir")
             .short("c")
             .long("config")
             .value_name("DIR")
             .help("Sets a custom config directory.")
             .takes_value(true)
             .default_value("/etc/moin-dm"))
        .arg(Arg::with_name("no-login")
             .long("no-login")
             .help("If set, moin-dm will assume the user is already logged in and just start the selected setup.")
             .takes_value(false))
        .get_matches();

    let config_dir = PathBuf::from(matches.value_of("config-dir").unwrap());
    let login = !matches.is_present("no-login");

    let mut instance = cli::ViewInstance::new(&config_dir);

    if login {
        match read_last_username(&config_dir) {
            Ok(name) =>  instance.add_login(&name),
            Err(_) =>    instance.add_login("<nobody>"),
        };
    }

    let mut setups: Vec<setups::Setup> = setups::available_setups(&config_dir).unwrap_or(Vec::new());
    setups.push(Default::default());
    instance.add_setups(setups);

    let selection = instance.run_interaction();
    test_selection(&selection, login).expect("Selection had missing answers.");

    if login {
        let username = selection.username().unwrap();
        if let Err(e) = save_last_username(&config_dir, username) {
            println!("{}", e);
        }
        setups::start_with_new_session(username, selection.setup().unwrap()).expect("Could not start setup.");
    } else {
        setups::start_with_existing_session(selection.setup().unwrap()).expect("Could not start setup.");
    }
}


fn test_selection(selection: &cli::Selection, login: bool) -> Result<()> {
    if !selection.has_setup() {
        Err(anyhow!("Setup was not set in selection."))
    } else if login && !selection.is_complete() {
        Err(anyhow!("Username was not set in selection"))
    } else {
        Ok(())
    }
}


fn read_last_username(config_dir: &Path) -> Result<String> {
    let mut file_path = PathBuf::from(config_dir);
    file_path.push(r"last_username");

    let mut file = File::open(file_path)?;
    let mut name_buf = Vec::new();
    file.read_to_end(&mut name_buf)?;

    let res = String::from_utf8(name_buf)?;

    Ok(res)
}

fn save_last_username(config_dir: &Path, new_username: &str) -> Result<()> {
    let mut file_path = PathBuf::from(config_dir);
    file_path.push(r"last_username");

    let mut file = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(file_path)?;

    let mut bytes_written = 0;
    while bytes_written < new_username.len() {
        bytes_written += file.write(new_username.as_bytes())?;
    }
    file.flush()?;

    Ok(())
}

