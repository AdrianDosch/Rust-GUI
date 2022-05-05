use std::path::Path;

use rust_gui::*;
use walkdir::WalkDir;

fn get_size(dir: &str) -> u64 {
    let total_size = WalkDir::new(dir)
        .min_depth(1)
        .max_depth(3)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.metadata().ok())
        .filter(|metadata| metadata.is_file())
        .fold(0, |acc, m| acc + m.len());

    total_size
}

fn main() {
    let gui = Gui::new("size calculator");
    let gui = gui.window(
        Window::new("window label")
            .add(Text::new(
                "This app calculates the size of a directory recursive up to layer three.",
            ))
            .add(Button::new("get Size:"))
            .same_line(InputText::new("###1"))
            .add(Text::new("input a Directory...")),
    );
    let gui = gui.build();

    let rec = gui.start();

    while gui.is_running() {
        rec.recv().unwrap(); //wait until one rendering loop has finished so the input got updated.

        if gui.get(0, Widget::Button(0)).unwrap() {
            let text;
            let input: String = gui.get(0, Widget::InputText(0)).unwrap();
            if Path::new(&input).exists() {
                gui.set(0, Widget::Text(1), String::from("calculating..."))
                    .unwrap();
                let byte_size: f32 = get_size(input.as_str()) as f32 / (1024.0 * 1024.0);
                text = format!("the directory is {} MB large.", byte_size);
            } else {
                text = String::from("directory not found");
            }
            gui.set(0, Widget::Text(1), text).unwrap();
        }
    }
}
