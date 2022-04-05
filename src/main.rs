use rust_imgui::*;
mod imgui;

fn hello_world() {
    println!("Hello World");
}

fn main() {
    let mut check_box = Checkbox::new("show demo window".into());
    check_box.set_callback(show_demo_window);
    
    let mut button = Button::new("click me".into());
    button.set_callback(hello_world);

    let color = Color::new();
    
    let text = Text::new("just some text".into());

    let mut window = Window::new("example window".into());
    window.append(&check_box);
    window.append(&button);
    window.append(&text);
    window.append(&color);

    let mut gui = GUI::new();
    gui.add_window(&window);

    while !gui.should_close() {
        gui.update();
    }
}
