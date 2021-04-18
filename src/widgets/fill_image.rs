use gdk_pixbuf::prelude::*;

use gtk::prelude::*;
use gtk::subclass::prelude::*;

use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Debug, Default, Clone)]
    pub struct FillImage {
        area: gtk::DrawingArea,
        pixbuf: RefCell<Option<gdk_pixbuf::Pixbuf>>,
    }

    impl FillImage {
        pub fn set_pixbuf(&self, pixbuf: gdk_pixbuf::Pixbuf) {
            let mut pb = self.pixbuf.borrow_mut();
            *pb = Some(pixbuf);
            self.area.queue_draw();
        }

        pub fn clear(&self) {
            let mut pb = self.pixbuf.borrow_mut();
            *pb = None;
            self.area.queue_draw();
        }

        fn draw(
            area: &gtk::DrawingArea,
            ctx: &cairo::Context,
            width: i32,
            height: i32,
        ) {
            // TODO: Clear the screen if no pixbuf
            let parent = match area.get_parent() {
                Some(p) => p,
                None => return,
            };

            let this = Self::from_instance(parent.downcast_ref().unwrap());

            let opt_pixbuf = this.pixbuf.borrow();
            let pixbuf = match opt_pixbuf.as_ref() {
                Some(p) => p,
                None => return,
            };

            let h_s = height as f64;
            let w_s = width as f64;

            let h_p = pixbuf.get_height() as f64;
            let w_p = pixbuf.get_width() as f64;

            let r_s = w_s / h_s;
            let r_p = w_p / h_p;

            let src_width;
            let src_height;

            if r_s > r_p {
                src_width = pixbuf.get_width();
                src_height = (r_s * w_p) as i32;
            } else {
                src_height = pixbuf.get_height();
                src_width = (r_s * h_p) as i32;
            }

            let src =
                pixbuf.new_subpixbuf(0, 0, src_width, src_height).unwrap();
            let scaled =
                src.scale_simple(width, height, gdk_pixbuf::InterpType::Hyper).unwrap();

            ctx.set_source_pixbuf(&scaled, 0., 0.);
            ctx.fill();
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FillImage {
        const NAME: &'static str = "FillImage";
        type Type = super::FillImage;
        type ParentType = gtk::Widget;
    }

    impl ObjectImpl for FillImage {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            self.area.set_parent(obj);
            self.area.set_hexpand(true);
            self.area.set_vexpand(true);
            self.area.set_draw_func(Self::draw);
        }

        fn dispose(&self, _: &Self::Type) {
            self.area.unparent();
        }
    }

    impl WidgetImpl for FillImage {
        fn get_request_mode(&self, _: &Self::Type) -> gtk::SizeRequestMode {
            self.area.get_request_mode()
        }

        fn size_allocate(
            &self,
            _: &Self::Type,
            width: i32,
            height: i32,
            baseline: i32,
        ) {
            self.area.allocate(width, height, baseline, None);
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
            let child = self.area.measure(orientation, for_size);
            *min = child.0;
            *nat = child.1;
            *min_base = child.2;
            *nat_base = child.3;
        }
    }
}

glib::wrapper! {
    pub struct FillImage(ObjectSubclass<imp::FillImage>)
        @extends gtk::Widget;
}

impl FillImage {
    pub fn new() -> Self {
        glib::Object::new(&[]).unwrap()
    }

    pub fn clear(&self) {
        let instance = imp::FillImage::from_instance(self);
        instance.clear();
    }
    pub fn set_pixbuf(&self, pixbuf: gdk_pixbuf::Pixbuf) {
        let instance = imp::FillImage::from_instance(self);
        instance.set_pixbuf(pixbuf);
    }
}
