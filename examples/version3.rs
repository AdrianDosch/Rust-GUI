use core::time;
use std::{thread, sync::{Arc, Mutex}, str::FromStr};

use rust_gui::*;

fn main() {
    let gui = Gui::new()
            .window(Window::new()
                .button(Button::new().callback(||{println!("pressed!")}))
                .text(Text::new("just some text"))
                .checkbox(Checkbox::new())
            ).build();
    
    gui.start();

    while gui.is_running() {
        // gui.lock().unwrap().set(0, Widget::Button(0), true);
        let x = *gui.blocking_read().get(0, Widget::Button(0)).unwrap();
        // let x = *gui.lock().unwrap().get(0, Widget::Checkbox(0)).unwrap();
        if x {
            println!("true");
            gui.blocking_write().set(0, Widget::Text(0), String::from("nice text")).unwrap();
        } else {}
    }
}