use std::{rc::Rc, cell::RefCell};

use rust_imgui::*;

fn hello_world() {
    println!("Hello World");
}

fn main() {
    let mut check_box = Checkbox::new("show demo window".into());
    check_box.borrow_mut().set_callback(show_demo_window);
    
    let mut button = Button::new("click me".into());
    button.borrow_mut().set_callback(hello_world);

    let color = Color::new("background".into());
    let text = Text::new("just some text".into());

    let mut window = Window::new("example window".into());
    // build_window!(
    //     // window,
    //     // &check_box,
    //     // &button,
    //     // &text,
    //     // &color
    // );

    
    window.add_component(color.clone());
    window.add_component(text.clone());
    window.add_component(button.clone());
    window.add_component(check_box.clone());
    
    let mut gui = GUI::new();
    gui.add_window(&window);

    while !gui.should_close() {
        gui.update(Some(color.borrow().col));
        if button.borrow().value {
            text.borrow_mut().text = "hi".into();
            button.borrow_mut().text = "you clicked me!".into();
            check_box.borrow_mut().value = true;
        }
    }
}
