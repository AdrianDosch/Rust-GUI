use lazy_static::__Deref;
use rust_gui::*;

fn main() {
    let gui = Gui::new().build();
    let gui = gui
        .window(
            Window::new()
                .add(Button::new("Button1"))
                .same_line(Text::new("label\ntestset"))
                .add(InputText::new("###1"))
                .same_line(Checkbox::new("print input"))
                .add(InputColor::new("choose a color"))
                .add(Button::new("button2")
                    .callback(move |gui: &Gui|{
                        gui.set(0, Widget::Text(0), String::from("clicked!\0")).unwrap();
                    })
                )
        );
    // let gui = gui.build();

    let receiver = gui.start();

    while gui.is_running() {
        //wait until one rendering loop has finished
        receiver.recv().unwrap();

        if gui.blocking_read().get(0, Widget::Button(0)).unwrap() {
            println!("Button 1");
        }

        if gui.blocking_read().get(0, Widget::Checkbox(0)).unwrap() {
            let x: String = gui.blocking_read().get(0, Widget::InputText(0)).unwrap();
            println!("{}", x);
        }
    }
}
