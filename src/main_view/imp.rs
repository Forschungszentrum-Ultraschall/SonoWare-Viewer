use std::{fs::File, io::BufReader};
use std::io::Read;
use crate::data;
use gtk::{subclass::prelude::*, CompositeTemplate, Button, glib::{self,
                                                                  subclass::InitializingObject},
          FileDialog, gio::Cancellable, prelude::FileExt, Window, gio};
use gtk::prelude::ButtonExt;

#[derive(CompositeTemplate, Default)]
#[template(resource = "/org/fzu/ui/main_view.xml")]
pub struct MainView {
    #[template_child]
    pub select_data: TemplateChild<Button>,
}

#[glib::object_subclass]
impl ObjectSubclass for MainView {
    const NAME: &'static str = "MainView";
    type Type = super::MainView;
    type ParentType = gtk::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for MainView {
    fn constructed(&self) {
        self.parent_constructed();

        self.select_data.connect_clicked(move |_| {
            let filters = gio::ListStore::new::<gtk::FileFilter>();
            let sonoware_filter = gtk::FileFilter::new();
            sonoware_filter.add_pattern("*.sdt");
            sonoware_filter.set_name(Some("Sonoware Messdaten"));
            filters.append(&sonoware_filter);

            let file_chooser = FileDialog::builder()
                .title("Wähle eine Messung aus")
                .accept_label("Öffnen")
                .filters(&filters)
                .build();

            let new_file = Window::builder().build();

            file_chooser.open(Some(&new_file), Cancellable::NONE, move |file| {
                match file {
                    Ok(file_dialog) => {
                        let file_path = file_dialog.path().expect("Couldn't get file path");
                        let file_data = File::open(file_path).expect("Failed to open file");

                        let mut reader = BufReader::new(file_data);
                        let mut data = Vec::new();
                        reader.read_to_end(&mut data).expect("Failed to read file content!");

                        let option_us_data = data::UsData::load_sonoware(data);

                        match option_us_data {
                            Some(us_data) => {
                                println!("{:?}", us_data.c_scan(0).unwrap());
                            }
                            None => {
                                println!("Failed to load sonoware data");
                            }
                        }
                    }
                    Err(error) => {
                        println!("Error: {}", error);
                    }
                }
            });
        });
    }
}

impl WidgetImpl for MainView {}
impl WindowImpl for MainView {}
impl ApplicationWindowImpl for MainView {}
