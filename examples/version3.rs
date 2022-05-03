use rust_gui::*;

fn main() {
    let gui = Gui::new()
        .window(
            Window::new()
                .button(Button::new("Button").callback(|| println!("pressed!")))
                .text(Text::new("just some text"))
                .checkbox(Checkbox::new("check"))
                .input_text(InputText::new("input"))
                .add(Button::new("B"))
                .add(Text::new("label"))
                .add(InputColor::new("choose a color"))
        )
        .build();

    let receiver = gui.start();

    while gui.is_running() {
        //wait until one rendering loop has finished
        receiver.recv().unwrap();

        if gui.get(0, Widget::Button(1)).unwrap() {
            println!("Button 1");
        }

        if gui.get(0, Widget::Button(0)).unwrap() {
            println!("Button 2");
            gui.set(0, Widget::Text(0), String::from("nice text"))
                .unwrap();
        }

        if gui.get(0, Widget::Checkbox(0)).unwrap() {
            let x: String = gui.get(0, Widget::InputText(0)).unwrap();
            println!("{}", x);
        }
    }
}
