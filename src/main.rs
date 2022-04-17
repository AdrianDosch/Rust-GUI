use std::{rc::Rc, cell::RefCell};

use rust_imgui::*;

fn main() {
    let check_box = Checkbox::new("show demo window".into());
    check_box.borrow_mut().set_callback(show_demo_window);

    let button = Button::new("click me".into());

    let color = Color::new("background".into());
    let text = Text::new("0".into());


    let counter = Rc::new(RefCell::new(0));
    
    let callback = enclose! { (text, counter) move || {
        *counter.borrow_mut() += 1;
        text.borrow_mut().text = format!("{}", counter.borrow_mut()).into();
    }};

    button.borrow_mut().set_callback(callback);
    

    let window = Window::new("example window".into());
    build_window!(window, check_box, button, text, color);

    let mut gui = GUI::new();
    gui.add_window(window.clone());

    while !gui.should_close() {
        gui.update(Some(color.borrow().col));
        if button.borrow().value {
            println!("{:?}", counter.borrow());
            button.borrow_mut().text = "you clicked me!".into();
        }
    }
}