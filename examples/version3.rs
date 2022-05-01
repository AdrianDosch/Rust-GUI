use std::{thread, sync::{Arc, Mutex}};

use rust_gui::*;

fn main() {
    let gui = Gui::new()
            .window(Window::new()
                .button(Button::new().callback(||{println!("pressed!")}))
            ).build();
    
    gui.start();
    // let gui_cp = gui.clone();

    // thread::spawn(move || {
    //     loop {
    //         gui_cp.lock().unwrap().update();
    //     }
    // });

    // loop {
    //     gui_cp.lock().unwrap().update();
    // }

    loop {
        // gui.lock().unwrap().set(0, Widget::Button(0), true);
    }
}