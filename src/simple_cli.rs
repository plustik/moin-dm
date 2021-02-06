
use std::{
    fs::File,
    io::{
        Error,
        ErrorKind,
        stdin,
        stdout,
        Read,
        Result,
        Write,
    },
    path::Path,
    str::FromStr,
};

use crate::setups::Setup;

pub fn run(mut setups: Vec<Setup>, config_dir: &Path) -> Result<Setup> {
    let mut output = stdout();

	let mut issue_location = config_dir.to_owned();
    issue_location.push(r"issue");
    if issue_location.is_file() {
        // Write issue file to stdout:
        let mut buf = Vec::new();
        let mut file = File::open(issue_location)?;
        file.read_to_end(&mut buf)?;
        output.write_all(&buf[..])?;
    }

    println!();

    let mut index = 0; 
    for setup in setups.iter() {
        println!("[{}]  {}", index, setup.name());
        index += 1;
    }

    let input = stdin();
    #[allow(unused_assignments)]
    let mut selected_index = 0;
    loop {
        println!("Choose setup: ");
        let mut line = String::new();
        if let Ok(2) = input.read_line(&mut line) {
            // Remove the \n from the line:
            line.pop();
            // Check weather to quit moin-dm:
            if line.as_str() == "q" || line.as_str() == "Q" {
                return Err(Error::new(ErrorKind::Interrupted, "The selection was quit.")); 
            }
            // check which Setup to start:
            if let Ok(i) = usize::from_str(&line) {
                selected_index = i;
                break;
            }
        }
    }

    Ok(setups.remove(selected_index))
}
