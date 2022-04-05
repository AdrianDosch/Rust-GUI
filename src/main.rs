use rust_imgui::*;
mod imgui;
// use imgui::show_demo_window;

fn main() {
    let mut window = Window::new();
    let mut gui = GUI::new();
    let mut check_box = Checkbox::new("show demo window".into());
    let mut button = Button::new("click me".into());

    button.set_callback(|| println!("test"));
    check_box.set_callback(||show_demo_window());
    window.append(check_box);
    window.append(button);
    window.append(Text::new("just some text".into()));
    gui.add_window(&window);

    while !gui.should_close() {
        gui.update();
        // println!("{:?}", window.items[1].get_value());
    }
}
