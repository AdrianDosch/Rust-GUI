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

    let mut gui = GUI::<Checkbox>::new();
    let mut window = Window::new();
    window.append(Checkbox::new("test".into()));
    gui.add_window(window);
    while !gui.should_close() {
        gui.update();
        println!("{:?}", gui.windows[0].items[0].get_value());
    }
}
