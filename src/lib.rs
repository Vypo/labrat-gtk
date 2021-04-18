extern crate libadwaita as adw;

mod bridge;
mod ptr;
mod secrets;
mod ui;
mod util;
mod widgets;

use crate::bridge::Bridge;
use crate::secrets::{Secrets, SecretsExt};
use crate::ui::root::Root;

use gettextrs::TextDomain;

use gtk::prelude::*;

use std::cell::RefCell;
use std::env::args;
use std::rc::Rc;

const QUALIFIER: &'static str = "dev.vypo.labrat-gtk";
const ORGANIZATION: &'static str = "Labrat";

pub fn main() {
    TextDomain::new("labrat")
        .push(std::env::current_dir().unwrap())
        .init()
        .ok();

    let application =
        gtk::Application::new(Some(QUALIFIER), Default::default())
            .expect("Initialization failed...");

    let bridge = Bridge::spawn();
    let root_ui: Rc<RefCell<Option<ptr::Owned<Root>>>> = Default::default();
    let weak = Rc::downgrade(&root_ui);

    let client = bridge.client();
    let util = crate::util::Util::new(client).unwrap();

    application.connect_activate(move |app| {
        let clone = util.client().clone();
        glib::MainContext::default().spawn(async move {
            // TODO: Make login screen and move getting secrets onto a thread.
            let secrets = Secrets::new().unwrap();
            let cookie = secrets.get().unwrap().unwrap();
            clone.login(&cookie).await.unwrap();
        });

        if let Some(cell) = weak.upgrade() {
            if cell.borrow().is_some() {
                return;
            }

            let root = Root::new(app, util.clone()).unwrap();
            root.show();

            *cell.borrow_mut() = Some(root);
        }
    });

    application.run(&args().collect::<Vec<_>>());
    drop(root_ui);
    bridge.join();
}
