
use anyhow::{Result, anyhow};
use cursive::Cursive;
use cursive::views::{TextView, Dialog, Button, LinearLayout, EditView, SelectView, DummyView, PaddedView};
use cursive::theme::{Theme, load_theme_file};
use cursive::traits::{Identifiable, Resizable};
use pam::Authenticator;
use users::all_users;

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::setups::Setup;


pub struct ViewInstance {
    cursive: Cursive,
    error_layer: Dialog,
    login_layer: Option<Dialog>,
    session_layer: Option<Dialog>,
}

impl ViewInstance {

    pub fn new(config_dir: &Path) -> Self {
       
        let mut curs = cursive::default();
        // setting theme:
        curs.set_theme(read_cursive_theme(&config_dir).unwrap_or(Theme::default()));
        curs.set_user_data(CursiveData::new());

        let err_view: Dialog = Dialog::around(TextView::new("Error!"))
                   .title("Moin")
                   .button("Quit", |siv| siv.quit());

        ViewInstance {
            cursive: curs,
            error_layer: err_view,
            login_layer: None,
            session_layer: None,
        }
    }

    pub fn add_login(&mut self, default_username: &str) {
       let description = LinearLayout::vertical()
           .child(TextView::new("User: "))
           .child(DummyView)
           .child(TextView::new("Password:  "));

       let user_view = LinearLayout::horizontal()
            .child(TextView::new(default_username)
               .with_name("username"))
            .child(DummyView.fixed_width(6))
            .child(Button::new("Change", enter_username_menu));
        
       let credentials = LinearLayout::vertical()
           .child(user_view)
           .child(DummyView)
           .child(EditView::new()
                  .secret()
                  .on_submit(|curs, _| login(curs))
                  .with_name("password")
                  .fixed_width(20));

       let content = LinearLayout::horizontal()
           .child(description)
           .child(credentials);

       self.login_layer = Some(Dialog::around(content)
           .title("Moin")
           .button("Login", login)
           .button("Quit", |siv| siv.quit()));

       // DEBUG:
       println!("Added login layer.");
    }

    pub fn add_setups(&mut self, available_setups: Vec<Setup>) {
        
        let mut setup_view = SelectView::<Setup>::new();
        available_setups.into_iter().for_each(|s| setup_view.add_item(String::from(s.name()), s));
        setup_view.set_on_submit(select_setup);

        let content = LinearLayout::vertical()
            .child(TextView::new("Select Setup: "))
            .child(DummyView)
            .child(setup_view);

        let new_layer = Dialog::around(PaddedView::lrtb(2, 2, 2, 2, content))
            .title("Moin")
            .button("Quit", |siv| siv.quit());
        self.session_layer = Some(new_layer);

       // DEBUG:
       println!("Added setup layer.");
    }

    pub fn run_interaction(mut self) -> Selection {

        let mut layers = Vec::new();
        layers.push(self.error_layer);
        self.session_layer.map(|l| layers.push(l));
        self.login_layer.map(|l| layers.push(l));

        self.cursive.add_layer(layers.pop().unwrap());
        self.cursive.focus_name("password").unwrap_or(());

        let data = CursiveData {
            next_layers: layers,
            selection: Selection::new(),
        };
        self.cursive.set_user_data(data);
    
        self.cursive.run();
        let data: CursiveData = self.cursive.take_user_data().unwrap();
        data.selection
    }
}


fn enter_username_menu(curs: &mut Cursive) {
    
    let mut names_view = SelectView::<String>::new();
    let users = unsafe { all_users() };
    users.filter(|u| u.uid() >= 1000)
        .map(|u| OsString::from(u.name()))
        .filter_map(|name_osstring| name_osstring.into_string().ok())
        .for_each(|name_string| names_view.add_item_str(name_string));
    names_view.set_on_submit(change_username);

    let content = LinearLayout::vertical()
        .child(TextView::new("Select user: "))
        .child(DummyView)
        .child(names_view);

    curs.add_layer(Dialog::around(PaddedView::lrtb(2, 2, 2, 2, content))
                   .title("Moin")
                   .button("Quit", |siv| siv.quit()));
    
}

fn change_username(curs: &mut Cursive, new_name: &str) {
    curs.call_on_name("username", |view: &mut TextView| view.set_content(new_name));    
    curs.focus_name("password").unwrap();
    curs.pop_layer();
}


fn login(curs: &mut Cursive) {

    let service = "moin-dm";
    let username = curs.call_on_name("username", |view: &mut TextView| view.get_content().source().to_string()).unwrap();
    let password = curs.call_on_name("password", |view: &mut EditView| view.get_content()).unwrap().to_string(); 

    let mut auth = Authenticator::with_password(service).unwrap();
    auth.close_on_drop = false;
    auth.get_handler().set_credentials(&username, password);

    if let Err(error) = auth.authenticate() {
        let error_view = LinearLayout::vertical()
            .child(TextView::new(format!("Authentication failed:\n{}", error)));
        curs.add_layer(Dialog::around(error_view)
                       .title("Error")
                       .button("Quit", |siv| siv.quit()));
    } else if let Err(error) = auth.open_session() {
        curs.add_layer(Dialog::around(TextView::new(format!("Opening session failed:\n{}", error)))
                       .title("Error")
                       .button("Quit", |siv| siv.quit()));
    } else { 
        // save username for later:
        let mut data: CursiveData = curs.take_user_data().unwrap();
        data.selection.set_username(username);
        let next_layer = data.next_layers.pop().unwrap();
        curs.set_user_data(data);

        // add next layer:
        curs.pop_layer();
        curs.add_layer(next_layer);
    }

}


fn select_setup(curs: &mut Cursive, selected_setup: &Setup) {

        let mut data: CursiveData = curs.take_user_data().unwrap();
        data.selection.set_setup(selected_setup.clone());

        curs.set_user_data(data);
        curs.quit();
}


pub struct CursiveData {
    next_layers: Vec<Dialog>,
    selection: Selection,
}

impl CursiveData {

    fn new() -> Self {
        CursiveData {
            next_layers: Vec::new(),
            selection: Selection::new(),
        }
    }
}


pub struct Selection {
    username: Option<String>,
    setup: Option<Setup>
}

impl Selection {

    pub fn new() -> Selection {
        Selection {
            username: None,
            setup: None
        }
    }

    pub fn username(&self) -> Option<&str> {
        match &self.username {
            Some(string) => Some(&string),
            None => None
        }
    }

    pub fn set_username<T: Into<String>>(&mut self, new_name: T) {
        self.username = Some(new_name.into());
    }

    pub fn setup(&self) -> Option<&Setup> {
        match &self.setup {
            Some(s) => Some(&s),
            None => None
        }
    }

    pub fn set_setup<T: Into<Setup>>(&mut self, new_setup: T) {
        self.setup = Some(new_setup.into());
    }


    pub fn is_complete(&self) -> bool {
        self.username.is_some() && self.setup.is_some()
    }

    pub fn has_setup(&self) -> bool {
        self.setup.is_some()
    }
}


fn read_cursive_theme(config_path: &Path) -> Result<Theme> {
    
    let mut path = PathBuf::from(config_path);
    path.push(r"theme.toml");

    match load_theme_file(&path) {
        Ok(theme)   => Ok(theme),
        Err(cursive::theme::Error::Io(_))   => Err(anyhow!("Could not read from file: {:?}", path)),
        Err(cursive::theme::Error::Parse(_))    =>Err( anyhow!("Could not parse file: {:?}", path)),
    }
}
