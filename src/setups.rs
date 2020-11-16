
use std::{env, os::unix::process::CommandExt, process::Command};
use std::ffi::OsString;
use std::io::{stdout, Write};
use std::path::{PathBuf, Path};

use crossterm::{QueueableCommand, terminal::{Clear, ClearType}, cursor::MoveTo};
use freedesktop_entry_parser::{parse_entry, Entry};
use users::{get_user_by_name, os::unix::UserExt};
use anyhow::{Result, anyhow};


pub fn start_session(username: &str, cmd_opt: Option<&str>) -> Result<()> {

    let user = get_user_by_name(&OsString::from(username)).ok_or(anyhow!("Could not get user."))?;

    env::set_current_dir(user.home_dir())?;

    // start the login shell:
    let mut child = Command::new(user.shell());
    child.uid(user.uid());
    child.gid(user.primary_group_id());
    // start as login shell:
    child.arg("-l");
    // start setup:
    if let Some(cmd_arg) = cmd_opt {
        child.arg(cmd_arg);
    }
    child.spawn()?.wait()?;

    Ok(())
}

pub fn start_with_new_session(username: &str, setup: &Setup) -> Result<()> {

    clear_terminal();

    start_session(username, setup.command())
}

pub fn start_with_existing_session(setup: &Setup) -> Result<()> {

    clear_terminal();

    if let Some(setup_cmd) = setup.command() {
        //split command to file and arguments:
        let mut parts = setup_cmd.split(' ');
        // execute setup command:
        let mut child = Command::new(parts.next().expect("The given argument contains no command."));
        child.args(parts);
        child.spawn().expect("Could not spawn setup process.").wait().expect("Setup process failed while running.");
    }

    Ok(())
}

fn clear_terminal() {
    let mut stdout = stdout();
    stdout.queue(Clear(ClearType::All)).expect("Could not queue Command.");
    stdout.queue(MoveTo(0, 0)).expect("Could not queue Command.");
    stdout.flush().expect("Could not flush commands.");
}


pub fn available_setups(config_dir: &Path) -> Result<Vec<Setup>> {
    
    let mut sessions_dir = PathBuf::from(config_dir); 
    sessions_dir.push(r"sessions");
    if !sessions_dir.is_dir() {
        return Err(anyhow!("Session directory does not exist."));
    }
    
    Ok(sessions_dir.read_dir().expect("Could not read directory.")
        .filter_map(|res| res.ok())
        .filter(|entry| entry.file_type().expect("Could not get file type.").is_file())
        .map(|entry| Setup::from(parse_entry(entry.path()).expect("Could not parse file."))).collect())
}


pub struct Setup {
    cmd: Option<String>,
    name: String
}

impl Setup {

    pub fn command(&self) -> Option<&str> {
        match &self.cmd {
            Some(string) => Some(&string),
            None    => None
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl From<Entry> for Setup {
    fn from(entry: Entry) -> Setup {

        let cmd_str = entry
            .section("Desktop Entry")
            .attr("Exec")
            .expect("Could not find attribute Exec.");
        let cmd = if cmd_str.is_empty() {
            None
        } else {
            Some(String::from(cmd_str))
        };

        let name_str = entry
            .section("Desktop Entry")
            .attr("Name")
            .expect("Could not find attribute Name.");
        let name = String::from(name_str);

        Setup {
            cmd,
            name
        }
    }
}

impl std::default::Default for Setup {
    fn default() -> Setup {
        Setup {
            cmd: None,
            name: String::from("Command Line")
        }
    }
}

impl Clone for Setup {
    fn clone(&self) -> Self {
        let cmd = match &self.cmd {
            Some(c) => Some(c.clone()),
            None => None
        };

        let name = self.name.clone();

        Setup {
            cmd,
            name
        }
    }
}
