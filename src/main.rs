// mod imgui;
// use imgui::*;

use rust_imgui::*;

fn main() {
    // let mut vars = Variables {
    //     window1: Window1 { show_demo_window: false, show_another_window: true },
    //     color: ImVec4 { x: 0.3, y: 0.3, z: 0.3, w: 1.0 }
    // };
    // let gui = GUI::new();
    // while !gui.terminate() {
    //     gui.update(&mut vars)
    // }

    let mut gui = GUI::new();
    let mut window = Window::new();
    let check_box = Checkbox::new("test".into());
    let button = Button::new("click me".into());
    window.append(check_box);
    window.append(button);
    window.append(Text::new("just some text".into()));
    gui.add_window(&window);
    while !gui.should_close() {
        gui.update();
        println!("{:?}", window.items[1].get_value());
    }
}
