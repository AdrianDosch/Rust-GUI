use std::{cell::RefCell, rc::Rc};

use rust_imgui::*;

fn def_win1<'a>() -> (
    Rc<RefCell<rust_imgui::Window<'a>>>,
    Rc<RefCell<i32>>,
    Rc<RefCell<rust_imgui::Button>>,
) {
    let check_box = Checkbox::new("show Dear ImGui demo window".into());
    check_box.borrow_mut().set_callback(show_demo_window);

    let button = Button::new("click me".into());

    let slider_int = SliderInt::new("".into());
    let slider_float = SliderFloat::new("##1".into());

    let color = Color::new("choose a color!".into());

    let text = Text::new("0".into());

    let counter = Rc::new(RefCell::new(0));
    let callback = enclose! { (text, counter) move || {
        *counter.borrow_mut() += 1;
        text.borrow_mut().label = format!("{}", counter.borrow_mut());
    }};
    button.borrow_mut().set_callback(callback);

    let window = Window::new("example window".into());
    build_window!(
        window,
        check_box,
        button,
        SameLine::new(None, None),
        text,
        color,
        slider_int,
        slider_float
    );
    (window, counter, button)
}

fn def_win2<'a>(name: &str, contents: &str) -> Rc<RefCell<Window<'a>>> {
    let text = Text::new(contents.into());
    let window = Window::new(name.into());
    build_window!(window, text);
    window
}
fn main() {
    let (window, counter, button) = def_win1();
    let window2 = def_win2("second window", "nice text");
    let window3 = def_win2("third window", "super nice text");

    let mut gui = GUI::new("example application".into());
    build_gui!(gui, window, window2);
    gui.add_window(window3);

    while !gui.should_close() {
        gui.update(None);

        //use widgets without callback functions
        if button.borrow().value {
            println!("value: {:?}", counter.borrow());
            button.borrow_mut().label = "you clicked me!".into();
        }
    }
}
