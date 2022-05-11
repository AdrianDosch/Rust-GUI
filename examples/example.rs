use rust_gui::*;

fn main() {
    let gui = Gui::new("example");
    // let gui = gui
    //     .window(
    //         Window::new("drag me!")
    //             .add(Button::new("Button1").callback(move |gui: &Gui| {
    //                 gui.set(0, Widget::Text(0), String::from("clicked!"))
    //                     .unwrap();
    //             }))
    //             .same_line(Text::new("click the button on the left\nsecond line!"))
    //             .add(InputText::new("###1"))
    //             .same_line(Checkbox::new("print input"))
    //             .add(InputColor::new("choose a color"))
    //             .add(Button::new("button2"))
    //             .add(SliderInt::new("slider").callback(|_: &Gui| println!("changed slider")))
    //             .add(SliderFloat::new("float slider"))
    //             .add(TreeNode::new("tree")
    //                 .add(Text::new("toller text"))
    //                 .add(Button::new("just a button")
    //                     .callback(|_: &Gui|{
    //                         println!{"pressed!"}
    //                     }))
    //             ),
    //     )
    //     .window(Window::new("second window")
    //                 .add(Checkbox::new("show demo window")
    //                     .callback(|gui: &Gui|{
    //                         let status = *gui.show_demo_window.blocking_read();
    //                         *gui.show_demo_window.blocking_write() = !status;
    //                     })));

    let gui = gui
        .window(
            Window::new("window label")
                .add(Button::new("button label").set_callback(|_: &Gui| println!("pressed!")))
                .add(
                    TreeNode::new("node").add(
                        Button::new("button").set_callback(|_: &Gui| println!("button in treenode")),
                    ),
                ),
        )
        .window(Window::new("win 2").add(Text::new("just some text")));

    let gui = gui.build();

    let receiver = gui.start();

    while gui.is_running() {
        receiver.recv().unwrap(); //wait until one rendering loop has finished so the input got updated.

        if gui.get::<Button, bool>(0, 0) {
            gui.set::<Text, String>(1, 0, String::from("new"));
        }

        let node = gui.get_widget::<TreeNode>(0, 0);
        if node.get_val::<Button, bool>(0) {
            println!("yay")
        }
    }
}
