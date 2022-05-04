use rust_gui::*;

fn main() {
    let gui = Gui::new("example");
    let gui = gui
        .window(
            Window::new("drag me!")
                .add(Button::new("Button1")
                    .callback(move |gui: &Gui|{
                        gui.set(0, Widget::Text(0), String::from("clicked!")).unwrap();
                    })    
                )
                .same_line(Text::new("click the button on the left\nsecond line!"))
                .add(InputText::new("###1"))
                .same_line(Checkbox::new("print input"))
                .add(InputColor::new("choose a color"))
                .add(Button::new("button2")
                )
        )
        .window(Window::new("second window"));
    let gui = gui.build();

    let receiver = gui.start();

    while gui.is_running() {
        receiver.recv().unwrap(); //wait until one rendering loop has finished so the input got updated.

        if gui.get(0, Widget::Button(1)).unwrap() {
            println!("Button 2");
        }

        if gui.get(0, Widget::Checkbox(0)).unwrap() {
            let x: String = gui.get(0, Widget::InputText(0)).unwrap();
            println!("{}", x);
        }
    }
}
