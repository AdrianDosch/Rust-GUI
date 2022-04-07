use rust_imgui::*;

fn hello_world() {
    println!("Hello World");
}

fn main() {
    let mut check_box = Checkbox::new("show demo window".into());
    check_box.set_callback(show_demo_window);
    
    let mut button = Button::new("click me".into());
    button.set_callback(hello_world);

    let color = Color::new("background".into());
    let text = Text::new("just some text".into());

    let mut window = Window::new("example window".into());
    build_window!(
        window,
        &check_box,
        &button,
        &text,
        &color
    );
    
    let mut gui = GUI::new();
    gui.add_window(&window);

    while !gui.should_close() {
        gui.update(Some(color.col));
    }
}
