use std::borrow::BorrowMut;

use rust_gui::*;

fn main() {
    
    
    let mut gui = GUI::new("file explorer".into());

    while !gui.should_close() {
        gui.update(None);
    }
}