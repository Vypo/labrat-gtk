use crate::ptr::{Owned, Ref, Wrap};

use gio::prelude::*;

use glib::signal::SignalHandlerId;

use gtk::prelude::*;

use std::borrow::Borrow;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug)]
pub struct PageListView<T> {
    _p: PhantomData<T>,
    end_diff: f64,
    list_store: gio::ListStore,
    multi_selection: gtk::MultiSelection,
    list_view: gtk::ListView,
    scroll_signal: RefCell<Option<(gtk::Adjustment, SignalHandlerId)>>,

    adding: AtomicUsize,
}

impl<T> PageListView<T>
where
    T: 'static + StaticType,
{
    pub(crate) fn new<C, F>(factory: &C, fetch: F) -> Owned<Self>
    where
        C: IsA<gtk::ListItemFactory>,
        F: 'static + Fn(Ref<Self>),
    {
        let fetch = Rc::new(fetch);

        let list_store = gio::ListStore::new(T::static_type());

        let multi_selection = gtk::MultiSelection::new(Some(&list_store));

        let list_view = gtk::ListViewBuilder::new()
            .factory(factory)
            .model(&multi_selection)
            .build();

        let owned = Owned::new(Self {
            _p: PhantomData,
            scroll_signal: Default::default(),
            multi_selection,
            list_store,
            list_view,
            end_diff: 200.,
            adding: AtomicUsize::new(0),
        });

        let weak = Owned::downgrade(&owned);

        owned.on_notify_vadjustment(&fetch);
        owned
            .list_view
            .connect_property_vadjustment_notify(move |_| {
                if let Some(this) = weak.upgrade() {
                    this.on_notify_vadjustment(&fetch);
                }
            });

        owned
    }
}

impl<T> Wrap<PageListView<T>>
where
    T: 'static,
{
    pub(crate) fn widget(&self) -> &impl IsA<gtk::Widget> {
        &self.list_view
    }

    pub fn len(&self) -> u32 {
        self.list_store.n_items()
    }

    fn on_notify_vadjustment<F>(&self, fetch: &Rc<F>)
    where
        F: 'static + Fn(Ref<PageListView<T>>),
    {
        if let Some((vadj, handler_id)) = self.scroll_signal.take() {
            vadj.disconnect(handler_id);
        }

        if let Some(vadj) = self.list_view.vadjustment() {
            let fetch_clone = fetch.clone();
            let weak = self.weak();

            let handler_id = vadj.connect_value_changed(move |vadj| {
                if let Some(this) = weak.upgrade() {
                    if 0 != this.adding.load(Ordering::SeqCst) {
                        // Don't start fetching if there are add loops running.
                        return;
                    }

                    let max = vadj.upper() - vadj.page_size();
                    if vadj.value() >= max - this.end_diff {
                        fetch_clone(this);
                    }
                }
            });

            self.scroll_signal.replace(Some((vadj, handler_id)));
        }
    }
}

impl<T> Wrap<PageListView<T>>
where
    T: 'static + StaticType + IsA<glib::Object>,
{
    pub fn last(&self) -> Option<T> {
        let len = self.len();
        if len == 0 {
            return None;
        }

        let obj = self.list_store.get_object(len - 1).unwrap();
        Some(obj.downcast().unwrap())
    }

    pub fn add<I>(&self, items: I)
    where
        I: 'static + IntoIterator,
        <I as IntoIterator>::Item: Borrow<T>,
    {
        self.adding.fetch_add(1, Ordering::SeqCst);

        let weak = self.weak();
        let mut iter = items.into_iter();

        glib::source::idle_add_local(move || {
            let this = match weak.upgrade() {
                Some(t) => t,
                None => return glib::Continue(false),
            };

            if let Some(item) = iter.next() {
                this.list_store.append(item.borrow());
                glib::Continue(true)
            } else {
                let val = this.adding.fetch_sub(1, Ordering::SeqCst);
                assert_ne!(val, 0);
                glib::Continue(false)
            }
        });
    }
}
