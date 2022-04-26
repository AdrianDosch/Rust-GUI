use std::path::Path;

use rust_gui::*;
use walkdir::WalkDir;

fn get_size(dir: &str) -> u64 {

    let total_size = WalkDir::new(dir)
    // .min_depth(1)
    // .max_depth(3)
    .into_iter()
    .filter_map(|entry| entry.ok())
    .filter_map(|entry| entry.metadata().ok())
    .filter(|metadata| metadata.is_file())
    .fold(0, |acc, m| acc + m.len());

    total_size
}

fn main() {
    let input = InputText::new("directory".into(), 255);
    let output = Text::new("input a directory path".into());

    let callback = enclose! { (input, output) move || {
        if Path::new(input.borrow().get_text()).exists() {
            let byte_size: f32 = get_size(input.borrow().get_text()) as f32 / (1024.0 * 1024.0);
            output.borrow_mut().label = format!("the directory is {} MB large.", byte_size);

        } else {
            output.borrow_mut().label = "input isn't a directory".into();
        }
    }};
    input.borrow_mut().set_callback(callback);

    let window = Window::new("app".into());
    build_window!(window, input, output);

    let mut gui = GUI::new("file explorer".into());
    gui.add_window(window);

    while !gui.should_close() {
        gui.update(None);
    }
}