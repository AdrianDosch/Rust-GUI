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

    let mut gui = GUI2::new("label")
        .add_window(Window2::new("label")
            .add(Text2::new()));

    loop {
        gui.update();
    }
    // println!("---end---");
}