
use std::{
    convert::TryFrom,
    fs::File,
    io::{Error, ErrorKind, Read, Result},
    path::{
        Path,
        PathBuf,
    },
    process::Command,
};

use serde::Deserialize;
use toml;


pub fn available_setups(config_dir: &Path) -> Result<Vec<Setup>> {
    
    let mut sessions_dir = PathBuf::from(config_dir); 
    sessions_dir.push(r"sessions");
    if !sessions_dir.is_dir() {
        return Err(Error::new(ErrorKind::Other, "The given directory does not exist."));
    }

    Ok(sessions_dir.read_dir()?
        .filter_map(|res| res.ok())
        .filter(|entry| entry.file_type().expect("Could not get file type.").is_file())
        .filter_map(|entry| Setup::try_from(entry.path().as_ref()).ok())
        .collect())
}


/// A Setup is a command, that can be choosen by the user and will thereupon be executed.
/// Each Setup is represented by a name and may have comment.
/// Setups are normaly parsed from a corresponding file in the config directory.
#[derive(Deserialize, Debug)]
pub struct Setup {
    command: String,
    name: String,
    comment: Option<String>,
}

impl Setup {

    /// Returns the name of the given Setup.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the command that will be executed to start this Setup.
    pub fn command(&self) -> Result<Command> {

        let mut parts = self.command.split(' ');

        let mut cmd = Command::new(parts.next().ok_or(Error::new(ErrorKind::Other, "The command string was empty."))?);
        cmd.args(parts);

        Ok(cmd)
    }

    /// Executes the command of this Setup and waits until it finishes.
    pub fn run(&self) -> Result<()> {
        self.command()?.status()?;

        Ok(())
    }
}

impl TryFrom<&Path> for Setup {
    type Error = Error;

    fn try_from(location: &Path) -> Result<Self> {

        let mut file = File::open(location)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;

        let setup: Setup = toml::from_slice(&buf[..])?;

        Ok(setup)
    }
}
