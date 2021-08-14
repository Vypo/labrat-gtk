use crate::bridge::errors::RequestError;
use crate::ptr::{Owned, Weak, Wrap};
use crate::util::Util;
use crate::widgets::{PageListView, FillImage};

use gio::prelude::*;

use gtk::prelude::*;
use gtk::subclass::prelude::*;

use labrat::keys::{SubmissionsKey, ViewKey};
use labrat::resources::{PreviewSize, Submission};

use once_cell::unsync::OnceCell;

use std::cell::{Cell, RefCell};

mod imp_item {
    use super::*;

    #[derive(Debug, Default, Clone)]
    pub struct ListSubmission(pub OnceCell<Submission>);

    #[glib::object_subclass]
    impl ObjectSubclass for ListSubmission {
        const NAME: &'static str = "ListSubmission";
        type Type = super::ListSubmission;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for ListSubmission {}
}

glib::wrapper! {
    pub struct ListSubmission(ObjectSubclass<imp_item::ListSubmission>);
}

impl ListSubmission {
    pub fn new(submission: Submission) -> Self {
        let new: Self = glib::Object::new(&[]).unwrap();
        let instance: &_ = imp_item::ListSubmission::from_instance(&new);
        instance.0.set(submission).unwrap();
        new
    }

    pub fn submission(&self) -> &Submission {
        imp_item::ListSubmission::from_instance(self)
            .0
            .get()
            .unwrap()
    }
}

#[derive(Debug)]
struct SubmissionListItemWidgets {
    box_: gtk::Box,
    thumbnail: FillImage,
    avatar: adw::Avatar,
}

mod imp_widget {
    use super::*;

    #[derive(Debug)]
    pub struct SubmissionListItem {
        widgets: SubmissionListItemWidgets,
        submission: RefCell<Option<Submission>>,
        weak: RefCell<Weak<Submissions>>,
    }

    impl Default for SubmissionListItem {
        fn default() -> Self {
            let thumbnail = FillImage::new();
            thumbnail.set_hexpand(true);
            thumbnail.set_vexpand(true);

            let avatar = adw::Avatar::new(32, None, false);

            let box_ = gtk::Box::new(gtk::Orientation::Vertical, 0);
            box_.append(&avatar);
            box_.append(&thumbnail);

            thumbnail.show();

            let widgets = SubmissionListItemWidgets {
                box_,
                thumbnail,
                avatar,
            };

            Self {
                widgets,
                submission: Default::default(),
                weak: RefCell::new(Weak::new()),
            }
        }
    }

    impl SubmissionListItem {
        pub fn set_weak(&self, weak: Weak<Submissions>) {
            self.weak.replace(weak);
        }

        fn same_submission(
            sub: &Submission,
            inst: &glib::WeakRef<super::SubmissionListItem>,
        ) -> Option<super::SubmissionListItem> {
            if let Some(inst) = inst.upgrade() {
                let this = Self::from_instance(&inst);
                let sub_key = ViewKey::from(sub);
                let cur_key =
                    this.submission.borrow().as_ref().map(ViewKey::from);
                if cur_key == Some(sub_key) {
                    return Some(inst);
                }
            }

            None
        }

        fn update(&self, submission: Submission) {
            self.widgets
                .avatar
                .set_text(Some(submission.artist().name()));

            if let Some(parent) = self.weak.borrow().upgrade() {
                // Fetch the avatar.
                let inst = self.instance().downgrade();
                let avatar_uri = submission.artist().avatar().to_string();

                let util = parent.util.clone();
                let sub0 = submission.clone();
                parent.util.spawn_local(async move {
                    let pixbuf = util.fetch_pixbuf(&avatar_uri).await?;

                    if let Some(inst) = Self::same_submission(&sub0, &inst) {
                        let this = Self::from_instance(&inst);
                        this.widgets.avatar.set_image_load_func(Some(
                            Box::new(move |_| Some(pixbuf.clone())),
                        ));
                    }

                    Result::<_, glib::Error>::Ok(())
                });

                // Fetch the thumbnail.
                let inst = self.instance().downgrade();
                let util = parent.util.clone();
                parent.util.spawn_local(async move {
                    let thumb = submission.preview(PreviewSize::Xl);
                    let pixbuf = util.fetch_pixbuf(thumb.as_str()).await?;

                    if let Some(inst) =
                        Self::same_submission(&submission, &inst)
                    {
                        // Only set the image iff the submission hasn't changed.
                        let this = Self::from_instance(&inst);
                        this.widgets.thumbnail.set_pixbuf(pixbuf);
                    }

                    Result::<_, glib::Error>::Ok(())
                });
            }
        }

        pub fn set_submission(&self, submission: Option<Submission>) {
            let old = self.submission.replace(submission.clone());

            match (old, submission) {
                (None, None) => (),
                (Some(_), None) => self.widgets.thumbnail.clear(),
                (None, Some(s)) => self.update(s),

                (Some(old), Some(new)) => {
                    let old_key = ViewKey::from(&old);
                    let new_key = ViewKey::from(&new);

                    if old_key != new_key {
                        self.update(new);
                    }
                }
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SubmissionListItem {
        const NAME: &'static str = "SubmissionListItem";
        type Type = super::SubmissionListItem;
        type ParentType = gtk::Widget;
    }

    impl ObjectImpl for SubmissionListItem {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            self.widgets.box_.set_parent(obj);
        }

        fn dispose(&self, _: &Self::Type) {
            self.widgets.box_.unparent();
        }
    }

    impl WidgetImpl for SubmissionListItem {
        fn get_request_mode(&self, _: &Self::Type) -> gtk::SizeRequestMode {
            gtk::SizeRequestMode::HeightForWidth
        }

        fn size_allocate(
            &self,
            _: &Self::Type,
            width: i32,
            height: i32,
            baseline: i32,
        ) {
            self.widgets.box_.allocate(width, height, baseline, None);
        }

        fn measure(
            &self,
            _: &Self::Type,
            orientation: gtk::Orientation,
            for_size: i32,
            min: &mut i32,
            nat: &mut i32,
            min_base: &mut i32,
            nat_base: &mut i32,
        ) {
            let child = self.widgets.box_.measure(orientation, for_size);
            *min_base = child.2;
            *nat_base = child.3;

            if orientation == gtk::Orientation::Vertical {
                // `for_size` is width.
                *min = for_size;
                *nat = for_size;
            } else {
                // `for_size` is height.
                *min = child.0;
                *nat = child.1;
            }
        }
    }
}

glib::wrapper! {
    pub struct SubmissionListItem(ObjectSubclass<imp_widget::SubmissionListItem>)
        @extends gtk::Widget;
}

impl SubmissionListItem {
    pub fn new() -> Self {
        glib::Object::new(&[]).unwrap()
    }

    pub fn set_submission(&self, submission: Option<Submission>) {
        let inner = imp_widget::SubmissionListItem::from_instance(self);
        inner.set_submission(submission);
    }

    pub fn set_weak(&self, weak: Weak<Submissions>) {
        let inner = imp_widget::SubmissionListItem::from_instance(self);
        inner.set_weak(weak);
    }
}

#[derive(Debug)]
pub struct Submissions {
    page_list_view: Owned<PageListView<ListSubmission>>,
    scrolled_window: gtk::ScrolledWindow,
    util: Util,
    fetching: Cell<bool>,
}

impl Submissions {
    pub(crate) fn new(util: Util) -> Owned<Self> {
        let factory = gtk::SignalListItemFactory::new();

        let page_list_view = PageListView::new(&factory, |plv| todo!());

        let scrolled_window = gtk::ScrolledWindow::new();
        scrolled_window.set_child(Some(page_list_view.widget()));

        let owned = Owned::new(Self {
            util,
            page_list_view,
            scrolled_window,
            fetching: Cell::new(false),
        });

        let weak = Owned::downgrade(&owned);

        factory.connect_setup(move |_, item| {
            let child = SubmissionListItem::new();
            child.set_weak(weak.clone());
            item.set_child(Some(&child));
        });

        factory.connect_bind(|_, list_item| {
            let child = list_item.child().unwrap();
            let item = list_item.item().unwrap();
            let widget: SubmissionListItem = child.downcast().unwrap();
            let sub: &ListSubmission = item.downcast_ref().unwrap();
            widget.set_submission(Some(sub.submission().clone()));
        });

        factory.connect_unbind(|_, item| {
            if let Some(child) = item.child() {
                let _widget: SubmissionListItem = child.downcast().unwrap();
                // Rows are bound/unbound as they are selected, so clearing the
                // binding doesn't make sense?
            }
        });

        factory.connect_teardown(|_, item| {
            item.set_child(Option::<&SubmissionListItem>::None);
        });

        owned
    }
}

impl Wrap<Submissions> {
    pub(crate) fn widget(&self) -> &impl IsA<gtk::Widget> {
        &self.scrolled_window
    }

    pub(crate) fn fetch(&self) {
        eprintln!("fetching");
        if self.fetching.replace(true) {
            return;
        }

        let this_weak = self.weak();

        eprintln!("pre-spawn");
        self.util.spawn_local::<_, RequestError>(async move {
            eprintln!("post-spawn");
            let this = match this_weak.upgrade() {
                Some(t) => t,
                None => return Ok(()),
            };

            let result = this
                .util
                .client()
                .submissions(SubmissionsKey::oldest())
                .await;
            this.fetching.replace(false);

            let submissions: Vec<_> = result?
                .into_items()
                .into_iter()
                .map(ListSubmission::new)
                .collect();

            eprintln!("got {} submissions", submissions.len());
            this.page_list_view.add(submissions);

            Ok(())
        });
    }
}
