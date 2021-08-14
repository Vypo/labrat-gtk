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
use std::rc::Rc;

const QUALIFIER: &'static str = "dev.vypo.labrat-gtk";
const ORGANIZATION: &'static str = "Labrat";

pub fn main() {
    TextDomain::new("labrat")
        .push(std::env::current_dir().unwrap())
        .init()
        .ok();

    let application =
        gtk::Application::new(Some(QUALIFIER), Default::default());

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
            secrets.set("b=90f6c627-b546-493b-a3f3-47cfb8a11107; a=94ad66cd-03fc-40c3-9f66-65539769fb23").unwrap();
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

    application.run();
    drop(root_ui);
    bridge.join();
}
