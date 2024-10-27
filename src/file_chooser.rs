use fltk::{app, prelude::*, window::Window, button::*, browser, dialog::{NativeFileChooser, FileDialogType, FileDialogAction}};
use std::rc::Rc;
use std::cell::RefCell;
use std::path::PathBuf;
use fltk::browser::MultiBrowser;

fn add_items_to_list(dialog_type: FileDialogType, list: &Rc<RefCell<MultiBrowser>>) {
    let mut dialog = NativeFileChooser::new(dialog_type);
    if let FileDialogAction::Success = dialog.try_show().unwrap() {
        for item in dialog.filenames() {
            list.borrow_mut().add(item.to_str().unwrap());
        }
    }
}

pub fn file_chooser_dialog() -> Vec<PathBuf> {
    let paths: Rc<RefCell<Vec<PathBuf>>> = Rc::new(RefCell::new(Vec::new()));

    let app = app::App::default().with_scheme(app::Scheme::Plastic);

    let window_width = 640;
    let window_height = 480;

    let mut window = Window::default()
        .center_screen()
        .with_size(window_width, window_height)
        .with_label(format!("Minicloud v{}", env!("CARGO_PKG_VERSION")).as_str());

    // Create a shared reference to the MultiBrowser
    let list = Rc::new(RefCell::new(
        browser::MultiBrowser::new(220, 10, 410, 460, "")
    ));

    let mut add_files_btn = Button::new(10, 10, 200, 50, "Add Files");
    add_files_btn.set_label_size(20);

    add_files_btn.set_callback({
        let list_clone = Rc::clone(&list); // Clone the Rc for the callback
        move |_| {
            add_items_to_list(FileDialogType::BrowseMultiFile, &list_clone);
        }
    });

    let mut add_directories_btn = Button::new(10, 70, 200, 50, "Add Directories");
    add_directories_btn.set_label_size(20);

    add_directories_btn.set_callback({
        let list_clone = Rc::clone(&list);
        move |_| {
            add_items_to_list(FileDialogType::BrowseMultiDir, &list_clone);
        }
    });

    let mut remove_selected_btn = Button::new(10, 150, 200, 50, "Remove Selected");
    remove_selected_btn.set_label_size(20);

    remove_selected_btn.set_callback({
        let list_clone = Rc::clone(&list);
        move |_| {
            let selected_items: Vec<i32> = list_clone.borrow().selected_items().to_vec();

            // Delete in reverse order to avoid problems with shifting indexes.
            for &item_line in selected_items.iter().rev() {
                println!("{}: {}", item_line, list_clone.borrow().text(item_line).unwrap());
                list_clone.borrow_mut().remove(item_line);
            }
        }
    });


    let mut start_server_btn = Button::new(10, 420, 200, 50, "Start Server");
    start_server_btn.set_label_size(20);

    start_server_btn.set_callback({
        let list_clone = list.clone(); // Clone the Rc for the callback
        let paths_clone = paths.clone();
        move |_| {
            for index in 1..list_clone.borrow().size() + 1 {
                let item = list_clone.borrow().text(index).unwrap();
                println!("{:?}", &item);
                paths_clone.borrow_mut().push(PathBuf::from(item));
            }
            app.quit();
        }
    });

    window.end();
    window.show();
    app.run().unwrap();

    paths.take()
}