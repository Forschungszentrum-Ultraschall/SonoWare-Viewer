mod imp;

use gtk::glib;

glib::wrapper! {
    pub struct AScanView(ObjectSubclass<imp::AScanView>)
        @extends gtk::Widget;
}
