// mod imgui;
// use imgui::*;

use rust_imgui::*;

fn main() {
    let mut gui = GUI::new();

    let mut window = Window::new();
    let check_box = Checkbox::new("test".into());
    let mut button = Button::new("click me".into());
    
    button.set_callback(||{println!("test")});
    window.append(check_box);
    window.append(button);
    window.append(Text::new("just some text".into()));
    gui.add_window(&window);

    while !gui.should_close() {
        gui.update();
        // println!("{:?}", window.items[1].get_value());
    }
}
