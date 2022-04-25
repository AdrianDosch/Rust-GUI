use std::{rc::Rc, cell::RefCell};

use rust_imgui::*;

fn main() {
    let check_box = Checkbox::new("show demo window".into());
    check_box.borrow_mut().set_callback(show_demo_window);

    let button = Button::new("click me".into());
    
    let slider_int = SliderInt::new("".into());
    let slider_float = SliderFloat::new("##1".into());

    let color = Color::new("background".into());
    
    let text = Text::new("0".into());

    let counter = Rc::new(RefCell::new(0));
    let callback = enclose! { (text, counter) move || {
        *counter.borrow_mut() += 1;
        text.borrow_mut().label = format!("{}", counter.borrow_mut());
    }};
    button.borrow_mut().set_callback(callback);

    let window = Window::new("example window".into());
    build_window!(window, check_box, button, SameLine::new(None, None), text, color, slider_int, slider_float);


    let text2 = Text::new("super toller text".into());
    let window2 = Window::new("window 2".into());
    build_window!(window2, text2);

    let mut gui = GUI::new();
    gui.add_window(window.clone());
    gui.add_window(window2.clone());

    while !gui.should_close() {
        gui.update(Some(color.borrow().value));
        if button.borrow().value {
            println!("{:?}", counter.borrow());
            button.borrow_mut().label = "you clicked me!".into();
        }
    }
}
