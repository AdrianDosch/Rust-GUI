use std::{str::FromStr, thread};

use rust_gui::*;

fn main() {
    println!("---start---");
    let mut gui = GUI2::new("winWinodw").add_window(
        Window2::new("window")
            .add_text(Text2::new("text"))
            .add_button(Button2::new("button"))
            .add_input_text(InputText2::new("input\0", 255)),
    );

    thread::spawn(enclose! { (gui) move || {
        let windows = &mut *gui.windows.lock().unwrap();
        let input = &mut windows[0].text_input.lock().unwrap()[0];
        let output = &mut windows[0].text.lock().unwrap()[0];
        {
            let mut ptr = output.label.lock().unwrap();
            *ptr = input.get_text();
        }
        {
            *output.label.lock().unwrap() = String::from_str(&input.get_text()).unwrap();
        }
    }});

    while !gui.should_close() {
        gui.update();
        let windows = &mut *gui.windows.lock().unwrap();
        let input = &mut windows[0].text_input.lock().unwrap()[0];
        let output = &mut windows[0].text.lock().unwrap()[0];
        {
            let mut ptr = output.label.lock().unwrap();
            *ptr = input.get_text();
        }
        {
            *output.label.lock().unwrap() = String::from_str(&input.get_text()).unwrap();
        }
        println!("{}", input.get_text());
    }
    println!("---end---");
}
