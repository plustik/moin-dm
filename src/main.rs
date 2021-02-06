
use std::{
    env::var_os,
    path::PathBuf,
};

extern crate anyhow;
extern crate clap;
use clap::{Arg, App};


mod setups;
mod simple_cli;
mod graphic_cli;


fn main() {

    let matches = App::new("moin-dm")
        .version("0.2")
        .author("Simeon Ricking <simeon.ricking@posteo.de>")
        .about("A command line interface for logging in and selecting a user session.")
        .arg(Arg::with_name("CONFIG-DIR")
             .short("c")
             .long("config")
             .value_name("DIR")
             .help("Sets a custom config directory.")
             .long_help(
                 "Sets the path to the config directory which contains the setups to choose from.
                 If this argument is not given, the first existing directory from the following list is choosen:
                 $XDG_CONFIG_HOME/moin-dm/
                 $HOME/.config/moin-dm/
                 /etc/moin-dm/
                 /usr/share/moin-dm/")
             .takes_value(true))
        .arg(Arg::with_name("INTERFACE")
             .short("i")
             .long("interface")
             .value_name("UI")
             .help("Sets the UI that is used.")
             .long_help(
                 "Sets the UI that is used to let the user choose the setup.")
             .takes_value(true)
             .default_value("simple")
             .possible_values(&["simple", "graphic"]))
        .get_matches();

    let config_dir = if let Some(path) = select_config_dir(matches.value_of("CONFIG-DIR").map(|str| PathBuf::from(str) )) {
        path
    } else {
        println!("The given directory does not exist.");
        return;
    };

    let setups: Vec<setups::Setup> = setups::available_setups(&config_dir).unwrap_or(Vec::new());

    let selection = match matches.value_of("INTERFACE").unwrap() {
        "simple" => {
            simple_cli::run(setups, &config_dir)
        },
        "graphic" => {
            graphic_cli::run(setups)
        },
        _ => {
            panic!("Unexpected value for argument 'inteface'.");
        },
    };
    match selection {
        Ok(s) => {
            // Start setup:
            if let Err(e) = s.run() {
                println!("{}", e);
            }
        },
        Err(e) => {
            println!("{}", e);
        },
    }
}


/// Returns the directory from which the Setups should be choosen:
fn select_config_dir(user_arg: Option<PathBuf>) -> Option<PathBuf> {

    // Return the directory given by the user, if it exists:
    if let Some(path) = user_arg {
        if path.is_dir() {
            return Some(path);
        }
    }

    // Return $XDG_CONFIG_HOME/moin-dm, if it exists:
    if let Some(config_home) = var_os("XDG_CONFIG_HOME") {
        let mut xdg_dir = PathBuf::from(config_home);
        xdg_dir.push(r"moin-dm");
        if xdg_dir.is_dir() {
            return Some(xdg_dir);
        }
    }

    // Return $HOME/.config/moin-dm, if it exists:
    if let Some(home_dir) = var_os("HOME") {
        let mut config_dir = PathBuf::from(home_dir);
        config_dir.push(r".config/moin-dm");
        if config_dir.is_dir() {
            return Some(config_dir);
        }
    }

    // Return /etc/moin-dm, if it exists:
    let etc_dir = PathBuf::from(r"/etc/moin-dm");
    if etc_dir.is_dir() {
        return Some(etc_dir);
    }

    // Return /usr/share/moin-dm, if it exists:
    let usr_dir = PathBuf::from(r"/usr/share/moin-dm");
    if usr_dir.is_dir() {
        return Some(usr_dir);
    }


    None
}
