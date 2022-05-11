use rust_gui::*;

fn main() {
    let gui = Gui::new("example");

    let gui = gui
        .window(
            Window::new("window label")
                .add(Button::new("button label").set_callback(|_: &Gui| println!("pressed!")))
                .same_line(Button::new("B"))
                .add(TreeNode::new("collapsable stuff")
                    .add(Button::new("button").set_callback(|_: &Gui| println!("button in tree node")))
                    .add(Text::new("just some text"))
                    .same_line(Checkbox::new("wow a checkbox"))
                )
                .add(SliderInt::new("i32"))
                .add(SliderFloat::new("f32"))
                .add(InputColor::new("choose a color"))
                .add(InputText::new("write some Text!"))
        )
        .window(
            Window::new("drag me!").add(Checkbox::new("show demo window").set_callback(|gui: &Gui| {
                let state = *gui.show_demo_window.blocking_read();
                *gui.show_demo_window.blocking_write() = !state;
            }))
            .add(Text::new("just some text")),
        );

    let gui = gui.build(); //get a handle to the gui which can be shared between different threads

    let receiver = gui.start(); //start the rendering loop of the gui in its own thread

    while gui.is_running() {
        receiver.recv().unwrap(); //wait until one rendering loop has finished so the input got updated.

        if gui.get::<Button, bool>(0, 0) { 
            gui.set::<Text, String>(1, 0, String::from("new text\n"));
        }

        let node = gui.get_widget::<TreeNode>(0, 0);
        if node.get_val::<Button, bool>(0) {
            println!("nice :)")
        }
    }
}
