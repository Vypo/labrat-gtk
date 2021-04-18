use crate::ptr::{Owned, Weak, Wrap};
use crate::util::Util;

use gettextrs::gettext;

use gio::prelude::*;

use gtk::prelude::*;

use std::cell::RefCell;

use super::root::Root;
use super::submissions::Submissions;

#[derive(Debug)]
pub struct Home {
    submissions: Owned<Submissions>,
    util: Util,

    notebook: gtk::Notebook,
}

impl Home {
    pub(crate) fn new(util: Util) -> Owned<Self> {
        let submissions = Submissions::new(util.clone());
        submissions.fetch();

        let notebook = gtk::NotebookBuilder::new().build();

        notebook.append_page(
            submissions.widget(),
            Some(&gtk::Label::new(Some(&gettext("Submissions")))),
        );

        Owned::new(Self {
            util,
            submissions,
            notebook,
        })
    }
}

impl Wrap<Home> {
    pub(crate) fn widget(&self) -> &impl IsA<gtk::Widget> {
        &self.notebook
    }
}
