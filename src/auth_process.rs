
use cursive::Cursive;
use cursive::views::{TextView, Dialog, Button, LinearLayout, EditView, SelectView, DummyView, ResizedView, PaddedView};
use cursive::traits::{Identifiable, Resizable};
use pam::{Authenticator};
use users::all_users;

use std::ffi::OsString;

use crate::setups::{Setup};


pub fn user_interaction(default_username: &str, available_setups: Vec<Setup>) -> Selection {

    let mut curs = cursive::default();

    let description = LinearLayout::vertical()
        .child(TextView::new("User: "))
        .child(DummyView)
        .child(TextView::new("Password:  "))
        .child(ResizedView::with_fixed_height(2, DummyView))
        .child(TextView::new("Setup: "));

    let user_view = LinearLayout::horizontal()
        .child(TextView::new(default_username)
               .with_name("username"))
        .child(DummyView.fixed_width(6))
        .child(Button::new("Change", enter_username_menu));

    let mut setup_view = SelectView::<Setup>::new();
    available_setups.into_iter().for_each(|s| setup_view.add_item(String::from(s.name()), s));
    setup_view.set_on_submit(enter);

    let credentials = LinearLayout::vertical()
        .child(user_view)
        .child(DummyView)
        .child(EditView::new()
               .secret()
               .with_name("password")
               .fixed_width(20))
        .child(ResizedView::with_fixed_height(2, DummyView))
        .child(setup_view);

    let content = LinearLayout::horizontal()
        .child(description)
        .child(credentials);

    curs.add_layer(Dialog::around(PaddedView::lrtb(2, 2, 2, 2, content))
                   .title("Login")
                   .button("Quit", |siv| siv.quit()));
    curs.focus_name("password").unwrap();

    curs.set_user_data(Selection::new());
    curs.run();

    curs.take_user_data().unwrap()
}

fn enter_username_menu(curs: &mut Cursive) {
    
    let mut names_view = SelectView::<String>::new();
    let users = unsafe { all_users() };
    users.map(|u| OsString::from(u.name())).filter_map(|name_osstring| name_osstring.into_string().ok()).for_each(|name_string| names_view.add_item_str(name_string));
    names_view.set_on_submit(change_username);

    curs.add_layer(Dialog::around(PaddedView::lrtb(2, 2, 2, 2, names_view))
                   .title("Select user:")
                   .button("Quit", |siv| siv.quit()));
    
}

fn change_username(curs: &mut Cursive, new_name: &str) {
    curs.call_on_name("username", |view: &mut TextView| view.set_content(new_name));    
    curs.pop_layer();
}

fn enter(curs: &mut Cursive, selected_setup: &Setup) {

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
        let mut selection: Selection = curs.take_user_data().unwrap();
        selection.set_username(username);
        selection.set_setup(selected_setup.clone());

        curs.set_user_data(selection);

        curs.quit();
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
}
