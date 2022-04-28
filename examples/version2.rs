use std::str::FromStr;

use rust_gui::*;

fn main() {
    // println!("---start---");

    // // let gui = GUI2::new("label")
    // //     .add_window(Window2::new("label")
    // //         .add(InputText2::new())
    // //         .add(Text2::new()))
    // //     .add_window(Window2::new("label")
    // //         .add(Text2::new())
    // //         .add(Button2::new("label")
    // //             .set_callback(||{})));

    let mut gui = GUI2::new("winWinodw").add_window(
        Window2::new("window")
            .add_text(Text2::new("text"))
            .add_button(Button2::new("button"))
            .add_input_text(InputText2::new("input\0", 255)),
    );

    while !gui.should_close() {
        gui.update();
        let input = &gui.windows[0].text_input[0];
        let output = &gui.windows[0].text[0];

        let mut ptr = output.label.lock().unwrap();
        *ptr = input.get_text();

        // output.label = String::from_str(input.get_text()).unwrap();

        // output.label = String::from_str(input.get_text()).unwrap();
        println!("{}", input.get_text());
    }
    // println!("---end---");
}
