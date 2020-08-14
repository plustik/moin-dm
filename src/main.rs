

mod setups;
mod auth_process;


fn main() {

    let mut setups: Vec<setups::Setup> = setups::available_setups().unwrap_or(Vec::new());
    setups.push(Default::default());

   let selection = auth_process::user_interaction("<nobody>", setups);

    if selection.is_complete() {
        setups::start_setup(selection.username().unwrap(), selection.setup().unwrap()).expect("Could not start setup.");
    } else {
        println!("Could not get selection.");
    }

}

