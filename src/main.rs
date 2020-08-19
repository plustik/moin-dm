
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{PathBuf, Path};

use anyhow::Result;


mod setups;
mod auth_process;


fn main() {

    #[allow(non_snake_case)]
    let CONFIG_DIR = PathBuf::from(r"/etc/moin-dm");


    let mut setups: Vec<setups::Setup> = setups::available_setups(&CONFIG_DIR).unwrap_or(Vec::new());
    setups.push(Default::default());

    let selection = match read_last_username(&CONFIG_DIR) {
        Ok(name) =>   auth_process::user_interaction(&name, setups),
        Err(_) =>       auth_process::user_interaction("<nobody>", setups),
    };

    if selection.is_complete() {
        let username = selection.username().unwrap();
        if let Err(e) = save_last_username(&CONFIG_DIR, username) {
            println!("{}", e);
        }
        setups::start_setup(username, selection.setup().unwrap()).expect("Could not start setup.");
    } else {
        println!("Could not get selection.");
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

