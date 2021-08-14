use crate::bridge::Client;
use crate::ptr::{Owned, Wrap};
use crate::util::Util;

use gettextrs::gettext;

use gio::prelude::*;

use gtk::prelude::*;

use super::home::Home;

#[derive(Debug)]
pub struct Root {
    home: Owned<Home>,

    titlebar: gtk::HeaderBar,
    window: gtk::ApplicationWindow,
    stack: gtk::Stack,

    back_btn: gtk::Button,
    back_action: gio::SimpleAction,

    util: Util,
}

impl Root {
    pub const BACK: &'static str = "win.back";

    pub fn new(
        application: &gtk::Application,
        util: Util,
    ) -> Result<Owned<Self>, glib::Error> {
        let home = Home::new(util.clone());

        let back_action = gio::SimpleAction::new("back", None);

        let back_btn = gtk::ButtonBuilder::new()
            .action_name(Self::BACK)
            .icon_name("go-previous")
            .build();

        back_action.set_enabled(false);

        let titlebar = gtk::HeaderBarBuilder::new().build();

        titlebar.pack_start(&back_btn);

        let stack = gtk::Stack::new();

        let window = gtk::ApplicationWindowBuilder::new()
            .application(application)
            .title(&gettext("First GTK Program"))
            .default_height(1440)
            .default_width(720)
            .child(&stack)
            .build();

        window.set_titlebar(Some(&titlebar));
        window.add_action(&back_action);

        let owned = Owned::new(Root {
            back_action: back_action.clone(),
            back_btn,
            titlebar,
            window,
            stack,
            home,
            util,
        });

        let weak = Owned::downgrade(&owned);
        back_action.connect_activate(move |_, _| {
            if let Some(root) = weak.upgrade() {
                root.pop();
            }
        });

        owned.push(owned.home.widget());

        Ok(owned)
    }
}

impl Wrap<Root> {
    fn pop(&self) {
        if let Some(child) = self.stack.visible_child() {
            self.stack.remove(&child);
            let pages = self.stack.pages();
            let len = pages.n_items();

            if len <= 1 {
                self.back_action.set_enabled(false);
            }

            if len > 0 {
                pages.select_item(len - 1, true);
            }
        }
    }

    pub fn push<P>(&self, child: &P)
    where
        P: IsA<gtk::Widget>,
    {
        self.stack.add_child(child);
        self.stack.set_visible_child(child);
        if self.stack.pages().n_items() > 1 {
            self.back_action.set_enabled(true);
        }
    }

    pub fn show(&self) {
        self.window.show();
    }

    pub fn titlebar(&self) -> &gtk::HeaderBar {
        &self.titlebar
    }

    pub fn window(&self) -> &gtk::ApplicationWindow {
        &self.window
    }
}
