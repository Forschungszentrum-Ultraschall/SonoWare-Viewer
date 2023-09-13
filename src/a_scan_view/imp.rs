use gtk::glib;
use gtk::subclass::prelude::*;

#[derive(Default)]
pub struct AScanView;

#[glib::object_subclass]
impl ObjectSubclass for AScanView {
    const NAME: &'static str = "AScanView";
    type Type = super::AScanView;
    type ParentType = gtk::Widget;
}

impl ObjectImpl for AScanView {}
impl WidgetImpl for AScanView {}
