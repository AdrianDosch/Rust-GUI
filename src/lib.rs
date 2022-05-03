mod backend;

use backend::*;
use std::{
    ffi::c_void,
    thread::{self, JoinHandle},
    time::{self, Duration}, sync::{Arc, mpsc::{self, Receiver}}, str::FromStr,
};
use tokio::sync::RwLock;

pub struct Gui {
    windows: Vec<Window>,
    glfw_window: RwLock<Option<&'static c_void>>,
    io: RwLock<Option<&'static c_void>>,
    should_close: RwLock<bool>,
    thread_handle: RwLock<Option<JoinHandle<()>>>,
}

impl Gui {
    pub fn new() -> Gui {
        Gui {
            windows: vec![],
            glfw_window: RwLock::new(None),
            io: RwLock::new(None),
            should_close: RwLock::new(false),
            thread_handle: RwLock::new(None),
        }
    }

    pub fn window(mut self, window: Window) -> Self {
        self.windows.push(window);
        self
    }

    pub fn set<T>(
        &self,
        window_idx: usize,
        widget: Widget,
        value: T,
    ) -> Result<(), WidgetNotFoundError>
    where
        Window: AccessWidget<T>,
    {
        if let Some(window) = self.windows.get(window_idx) {
            AccessWidget::set(window, widget, value);
            Ok(())
        } else {
            Err(WidgetNotFoundError)
        }
    }

    pub fn get<T>(&self, window_idx: usize, widget: Widget) -> Result<T, WidgetNotFoundError>
    where
        Window: AccessWidget<T>,
    {
        if let Some(window) = self.windows.get(window_idx) {
            Ok(AccessWidget::get(window, widget))
        } else {
            Err(WidgetNotFoundError)
        }
    }

    pub fn build(self) -> GuiHandle {
        Arc::new(self)
    }

    pub fn should_close(&self) -> bool {
        unsafe {
            if close_window(self.glfw_window.blocking_read().unwrap()) {
                return true;
            }
        }
        false
    }
}

pub type GuiHandle = Arc<Gui>;
pub trait Start {
    fn start(&self) -> Receiver<()>;
    fn is_running(&self) -> bool;
}

impl Start for GuiHandle {
    fn start(&self) -> Receiver<()>{
        let cp = self.clone();
        let (tx, rx) = mpsc::channel();
        let handle = thread::spawn(move || {
            unsafe {
                let window_handle = init_gui("test_label".as_ptr());
                let mut x = cp.glfw_window.blocking_write();
                *x = Some(window_handle.window);

                let mut io = cp.io.blocking_write();
                *io = Some(window_handle.io);
            }

            while !cp.should_close() {
                // let start_time = time::Instant::now();
                cp.update();
                tx.send(()).unwrap();
                // let time_delta = time::Instant::now() - start_time;
                // println!("{:?}", time_delta);
            }
        });
        let mut h = self.thread_handle.blocking_write();
        *h = Some(handle);
        rx
    }

    fn is_running(&self) -> bool {
        let x = Arc::strong_count(self);
        x > 1 
        
    }
}

impl Update for Gui {
    fn update(&self) {
        unsafe { start_frame() }
        for window in &self.windows {
            window.update();
        }
        let clear_color = ImGui_Vec4 {
            x: 0.3,
            y: 0.3,
            z: 0.3,
            w: 1.0,
        };

        unsafe {
            end_frame(
                self.glfw_window.blocking_read().unwrap(),
                self.io.blocking_read().unwrap(),
                clear_color,
            );
        }
    }
}

#[derive(Debug, Clone)]
pub struct WidgetNotFoundError;

#[derive(Debug)]
pub enum Widget {
    Button(usize),
    Checkbox(usize),
    Text(usize),
    InputText(usize),
}

pub struct Window {
    buttons: Vec<Arc<RwLock<Button>>>,
    text: Vec<Arc<RwLock<Text>>>,
    checkboxes: Vec<Arc<RwLock<Checkbox>>>,
    text_input: Vec<Arc<RwLock<InputText>>>,
    input_color: Vec<Arc<RwLock<InputColor>>>,
    widgets: Vec<Arc<RwLock<dyn Update + Send + Sync>>>,
}

impl Window {
    pub fn new() -> Self {
        Window {
            buttons: vec![],
            text: vec![],
            checkboxes: vec![],
            text_input: vec![],
            input_color: vec![],
            widgets: vec![],
        }
    }

    pub fn button(mut self, button: Button) -> Self {
        self.buttons.push(Arc::new(RwLock::new(button)));
        self
    }

    pub fn checkbox(mut self, checkbox: Checkbox) -> Self {
        self.checkboxes.push(Arc::new(RwLock::new(checkbox)));
        self
    }

    pub fn text(mut self, text: Text) -> Self {
        self.text.push(Arc::new(RwLock::new(text)));
        self
    }

    pub fn input_text(mut self, text_input: InputText) -> Self {
        self.text_input.push(Arc::new(RwLock::new(text_input)));
        self
    }

    // pub fn add<T: Update + 'static + Send + Sync>(mut self, widget: T) -> Self {
    //     self.widgets.push(RwLock::new(Box::new(widget)));
    //     self
    // }
}

pub trait Add<T> {
    fn add(self, widget: T) -> Self;    
}

impl Add<Checkbox> for Window {
    fn add(mut self, widget: Checkbox) -> Self {
        self.checkboxes.push(Arc::new(RwLock::new(widget)));
        self
    }
}

impl Add<InputText> for Window {
    fn add(mut self, widget: InputText) -> Self {
        self.text_input.push(Arc::new(RwLock::new(widget)));
        self
    }
}

impl Add<Text> for Window {
    fn add(mut self, widget: Text) -> Self {
        self.text.push(Arc::new(RwLock::new(widget)));
        self
    }
}

impl Add<Button> for Window {
    fn add(mut self, widget: Button) -> Self {
        self.buttons.push(Arc::new(RwLock::new(widget)));
        self
    }
}


fn to_update<T: Update + 'static + Send + Sync>(test: &Arc<RwLock<T>>) -> Arc<RwLock<dyn Update + 'static + Send + Sync>> {
    let x = test.clone();
    x
}

impl Add<InputColor> for Window {
    fn add(mut self, widget: InputColor) -> Self {
        let widget = Arc::new(RwLock::new(widget));       
        self.widgets.push(to_update(&widget));
        self
    }
}

pub trait AccessWidget<T> {
    fn set(&self, widget: Widget, value: T);
    fn get(&self, widget: Widget) -> T;
}

pub trait AccessWidget2<T, U> {
    fn set(&self, idx: usize, value: U);
    fn get(&self, idx: usize) -> U;
}

impl AccessWidget<bool> for Window {
    fn set(&self, widget: Widget, value: bool) {
        match widget {
            Widget::Button(i) => self
                .buttons
                .get(i)
                .expect("there aren't enough buttons")
                .blocking_write()
                .set(value),
            Widget::Checkbox(i) => self
                .checkboxes
                .get(i)
                .expect("msg")
                .blocking_write()
                .set(value),
            _ => unreachable!("set widget: {:?} not implemented", widget),
        }
    }

    fn get(&self, widget: Widget) -> bool {
        match widget {
            Widget::Button(i) => {
                self.buttons
                    .get(i)
                    .expect("there aren't enough buttons")
                    .blocking_read()
                    .value
            }
            Widget::Checkbox(i) => self.checkboxes.get(i).expect("msg").blocking_read().value,
            _ => unreachable!("set widget: {:?} not implemented", widget),
        }
    }
}

impl AccessWidget<String> for Window {
    fn set(&self, widget: Widget, value: String) {
        match widget {
            Widget::Text(i) => self
                .text
                .get(i)
                .expect("there aren't enough text widgets")
                .blocking_write()
                .set(value),
            Widget::InputText(i) => todo!(),
            _ => unreachable!("set widget: {:?} not implemented", widget),
        }
    }

    fn get(&self, widget: Widget) -> String {
        match widget {
            Widget::Text(i) => self
                .text
                .get(i)
                .expect("there aren't enough text widgets")
                .blocking_read()
                .value
                .clone(),
            Widget::InputText(i) => self
                .text_input
                .get(i)
                .expect("not enough input text")
                .blocking_read()
                .get_text()
                .to_string(),
            _ => unreachable!("set widget: {:?} not implemented", widget),
        }
    }
}

pub trait Update {
    fn update(&self);
}

impl Update for Window {
    fn update(&self) {
        unsafe { ImGui_Begin("windowlabel\0".as_ptr(), &true, 0) }
        for button in &self.buttons {
            button.blocking_write().update();
        }
        for text in &self.text {
            text.blocking_write().update()
        }
        for checkbox in &self.checkboxes {
            checkbox.blocking_write().update();
        }
        for text_input in &self.text_input {
            text_input .blocking_write().update();
        }
        for widget in &self.widgets {
            widget.blocking_write().update();
        }
        for color in &self.input_color {
            color.blocking_write().update();
        }
        unsafe { ImGui_End() }
    }
}

impl Update for Button {
    fn update(&self) {
        unsafe { ImGui_Button(self.label.as_ptr(), &self.value) }
        if self.value {
            (self.callback)();
        }
    }
}

impl Update for Text {
    fn update(&self) {
        unsafe { ImGui_Text(self.value.as_ptr()) }
    }
}

impl Update for Checkbox {
    fn update(&self) {
        unsafe { ImGui_Checkbox(self.label.as_ptr(), &self.value) }
    }
}

impl Update for InputText {
    fn update(&self) {
        unsafe { ImGui_InputText(self.label.as_ptr(), self.value.as_ptr(), self.buffersize, 0) }
    }
}

impl Update for InputColor {
    fn update(&self) {
        unsafe { ImGui_ColorEdit3(self.label.as_ptr(), &self.value) }
    }
}

pub trait Set<T> {
    fn set(&mut self, value: T);
}

impl Set<bool> for Button {
    fn set(&mut self, value: bool) {
        self.value = value;
    }
}

impl Set<String> for Text {
    fn set(&mut self, value: String) {
        self.value = value;
    }
}

impl Set<bool> for Checkbox {
    fn set(&mut self, value: bool) {
        self.value = value;
    }
}

impl Set<Vec<f32>> for InputColor {
    fn set(&mut self, value: Vec<f32>) {
        self.value = ImGui_Vec4 {x: value[0], y: value[1], z: value[2], w: value[3]};
    }
}

pub struct Button {
    label: String,
    value: bool,
    callback: Box<fn()>,
}

impl Button {
    pub fn new(label: &str) -> Self {
        let mut label = String::from_str(label).unwrap();
        if !label.ends_with("\0"){
            label.push('\0');
        };
        Button {
            label,
            value: false,
            callback: Box::new(|| {}),
        }
    }

    pub fn callback(mut self, callback: fn()) -> Self {
        self.callback = Box::new(callback);
        self
    }
}


pub struct Text {
    pub value: String,
}

impl Text {
    pub fn new(label: &str, ) -> Self {
        let mut label = String::from_str(label).unwrap();
        if !label.ends_with("\0"){
            label.push('\0');
        };
        Text {
            value: label,
        }
    }
}

pub struct Checkbox {
    label: String,
    pub value: bool,
}

impl Checkbox {
    pub fn new(label: &str) -> Self {
        let mut label = String::from_str(label).unwrap();
        if !label.ends_with("\0"){
            label.push('\0');
        };
        Checkbox { label, value: false }
    }
}

pub struct InputText {
    label: String,
    value: String,
    buffersize: i32,
}

impl InputText {
    pub fn new(label: &str) -> Self {
        let mut label = String::from_str(label).unwrap();
        if !label.ends_with("\0"){
            label.push('\0');
        };

        let mut string = String::new();
        for _ in 0..255 {
            string.push('\0');
        }
        InputText {
            label,
            value: string,
            buffersize: 255,
        }
    }

    pub fn get_text(&self) -> &str {
        let null_terminator_position = self.value.find('\0').unwrap();

        // if a truncation of a non ascii character occurred the string has a invalid memory layout past the intended end.
        &self.value[..null_terminator_position]
    }
}

pub struct InputColor {
    label: String,
    value: ImGui_Vec4,
}

impl InputColor {
    pub fn new(label: &str) -> InputColor {
        let mut label = String::from_str(label).unwrap();
        if !label.ends_with("\0"){
            label.push('\0');
        };

        let value = ImGui_Vec4 {
            x: 0.3,
            y: 0.3,
            z: 0.3,
            w: 1.0,
        };
        InputColor { value, label }
    }
}















// pub struct GUI<'a> {
//     pub windows: Vec<Rc<RefCell<Window<'a>>>>,
//     glfw_window: &'static c_void,
//     io: &'static c_void,
//     should_close: bool,
// }

// #[macro_export]
// //macro from webplatform
// macro_rules! enclose {
//     ( ($( $x:ident ),*) $y:expr ) => {
//         {
//             $(let $x = $x.clone();)*
//             $y
//         }
//     };
// }

// impl<'a> GUI<'a> {
//     pub fn new(window_label: String) -> GUI<'a> {
//         let mut window_label = window_label.clone();
//         if window_label.len() == 0 {
//             window_label.push(' ');
//         }
//         window_label.push('\0');
//         unsafe {
//             let window_handle = init_gui(window_label.as_ptr());
//             GUI {
//                 windows: vec![],
//                 glfw_window: window_handle.window,
//                 io: window_handle.io,
//                 should_close: false,
//             }
//         }
//     }

//     pub fn add_window(&mut self, window: Rc<RefCell<Window<'a>>>) {
//         self.windows.push(window);
//     }

//     pub fn update(&mut self, color: Option<ImGui_Vec4>) {
//         unsafe {
//             start_frame();
//         }
//         for i in 0..self.windows.len() {
//             self.windows[i].borrow().render();
//         }
//         unsafe {
//             let clear_color = if let Some(color) = color {
//                 color
//             } else {
//                 ImGui_Vec4 {
//                     x: 0.3,
//                     y: 0.3,
//                     z: 0.3,
//                     w: 1.0,
//                 }
//             };
//             end_frame(self.glfw_window, self.io, clear_color.clone());
//         }
//     }

//     pub fn should_close(&self) -> bool {
//         unsafe {
//             if close_window(self.glfw_window) {
//                 return true;
//             }
//         }
//         self.should_close
//     }
// }

// impl<'a> Drop for GUI<'a> {
//     fn drop(&mut self) {
//         unsafe {
//             destroy_gui(self.glfw_window);
//         }
//     }
// }

// #[macro_export]
// macro_rules! build_gui {
//     ($a:expr, $b:expr) => {
//         $a.add_window($b.clone());
//     };

//     ($a:expr, $b:expr, $c:expr) => {
//         $a.add_window($b.clone());
//         $a.add_window($c.clone());
//     };

//     ($a:expr, $b:expr, $($c:tt)*) => {
//         $a.add_window($b.clone());
//         build_window!($a,$($c)*)
//     };
// }

// pub struct Window<'a> {
//     pub items: Vec<&'a dyn ImGuiGlue>,
//     pub show: bool,
//     pub name: String,
//     pub components: Vec<Rc<RefCell<dyn ImGuiGlue>>>,
//     id: u32,
// }

// lazy_static::lazy_static! {
//     static ref UNIQUE_WINDOW_ID: Mutex<u32> = Mutex::new(0u32);
// }

// impl<'a> Window<'a> {
//     pub fn new(name: String) -> Rc<RefCell<Window<'a>>> {
//         let mut id = UNIQUE_WINDOW_ID.lock().unwrap();
//         let window = Rc::new(RefCell::new(Window {
//             items: vec![],
//             show: true,
//             name,
//             components: vec![],
//             id: id.clone(),
//         }));
//         *id += 1;
//         window
//     }

//     pub fn append<T: ImGuiGlue + 'static>(&mut self, comp: Rc<RefCell<T>>) {
//         self.components.push(comp);
//     }
// }

// #[macro_export]
// macro_rules! build_window {
//     ($a:expr, $b:expr) => {
//         $a.borrow_mut().append($b.clone());
//     };

//     ($a:expr, $b:expr, $c:expr) => {
//         $a.borrow_mut().append($b.clone());
//         $a.borrow_mut().append($c.clone());
//     };

//     ($a:expr, $b:expr, $($c:tt)*) => {
//         $a.borrow_mut().append($b.clone());
//         build_window!($a,$($c)*)
//     };
// }

// impl<'a> ImGuiGlue for Window<'a> {
//     fn render(&self) {
//         if self.show {
//             let mut name = self.name.clone();
//             // if name.is_empty() {
//             //     name = " ".into();
//             // }
//             name.push_str("###");
//             name.push_str(&self.id.to_string());
//             name.push('\0');
//             unsafe { ImGui_Begin(name.as_ptr(), &self.show, 0) }
//             for item in &self.items {
//                 item.render();
//             }
//             for item in &self.components {
//                 item.borrow().render();
//             }
//             unsafe { ImGui_End() }
//         }
//     }
// }

// #[derive(Callback, ImGuiGlue)]
// pub struct Checkbox {
//     pub label: String,
//     pub value: bool,
//     pub callback: Box<dyn Fn()>,
// }

// impl Checkbox {
//     pub fn new(label: String) -> Rc<RefCell<Checkbox>> {
//         Rc::new(RefCell::new(Checkbox {
//             label,
//             value: false,
//             callback: Box::new(|| {}),
//         }))
//     }
// }

// #[derive(ImGuiGlue)]
// pub struct Text {
//     pub label: String,
// }

// impl Text {
//     pub fn new(label: String) -> Rc<RefCell<Text>> {
//         Rc::new(RefCell::new(Text { label }))
//     }
// }

// #[derive(Callback, ImGuiGlue)]
// pub struct Button {
//     pub label: String,
//     pub value: bool,
//     callback: Box<dyn Fn()>,
// }

// impl Button {
//     pub fn new(label: String) -> Rc<RefCell<Button>> {
//         Rc::new(RefCell::new(Button {
//             label,
//             value: false,
//             callback: Box::new(|| {}),
//         }))
//     }
// }

// #[derive(ImGuiGlue)]
// pub struct Color {
//     label: String,
//     pub value: ImGui_Vec4,
// }

// impl Color {
//     pub fn new(label: String) -> Rc<RefCell<Color>> {
//         let value = ImGui_Vec4 {
//             x: 0.3,
//             y: 0.3,
//             z: 0.3,
//             w: 1.0,
//         };
//         Rc::new(RefCell::new(Color { value, label }))
//     }
// }

// #[derive(ImGuiGlue)]
// pub struct SameLine {
//     offset_from_start_x: f32,
//     spacing: f32,
// }

// impl SameLine {
//     pub fn new(offset_from_start_x: Option<f32>, spacing: Option<f32>) -> Rc<RefCell<Self>> {
//         let offset_from_start_x = if let Some(x) = offset_from_start_x {
//             x
//         } else {
//             0.0
//         };
//         let spacing = if let Some(x) = spacing { x } else { -1.0 };

//         Rc::new(RefCell::new(SameLine {
//             offset_from_start_x,
//             spacing,
//         }))
//     }
// }

// pub fn show_demo_window() {
//     unsafe { backend::show_demo_window() }
// }

// #[derive(Callback, ImGuiGlue)]
// pub struct SliderInt {
//     pub label: String,
//     pub value: i32,
//     callback: Box<dyn Fn()>,
//     pub min: i32,
//     pub max: i32,
// }

// impl SliderInt {
//     pub fn new(label: String) -> Rc<RefCell<Self>> {
//         Rc::new(RefCell::new(SliderInt {
//             label,
//             value: 0,
//             min: 0,
//             max: 100,
//             callback: Box::new(|| {}),
//         }))
//     }
// }

// #[derive(Callback, ImGuiGlue)]
// pub struct SliderFloat {
//     pub label: String,
//     pub value: f32,
//     callback: Box<dyn Fn()>,
//     pub min: f32,
//     pub max: f32,
// }

// impl SliderFloat {
//     pub fn new(label: String) -> Rc<RefCell<Self>> {
//         Rc::new(RefCell::new(SliderFloat {
//             label,
//             value: 0.0,
//             min: 0.0,
//             max: 1.0,
//             callback: Box::new(|| {}),
//         }))
//     }
// }

// #[derive(Callback)]
// pub struct InputText {
//     pub label: String,
//     value: String,
//     callback: Box<dyn Fn()>,
//     buffer_size: i32,
//     pub flags: i32,
// }

// impl ImGuiGlue for InputText {
//     fn render(&self) {
//         let prev = self.value.clone();

//         let mut label = self.label.clone();
//         if label.len() == 0 {
//             label.push(' ');
//         }
//         label.push('\0');

//         unsafe {
//             ImGui_InputText(
//                 label.as_ptr(),
//                 self.value.as_ptr(),
//                 self.buffer_size,
//                 self.flags,
//             );
//         }

//         if prev != self.value {
//             self.call_callback();
//         }
//     }
// }

// impl InputText {
//     pub fn new(label: String, size: i32) -> Rc<RefCell<Self>> {
        // let mut string = String::new();
        // for _ in 0..size {
        //     string.push('\0');
        // }

//         Rc::new(RefCell::new(InputText {
//             label,
//             value: string,
//             buffer_size: size,
//             flags: 0,
//             callback: Box::new(|| {}),
//         }))
//     }

//     pub fn get_text(&self) -> &str {
//         let null_terminator_position = self.value.find('\0').unwrap();

//         // if a truncation of a non ascii character occurred the string has a invalid memory layout past the intended end.
//         &self.value[..null_terminator_position]
//     }
// }

// pub trait Callback {
//     fn set_callback(&mut self, callback: impl Fn() + 'static);
//     fn call_callback(&self);
// }

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

// #[derive(Clone)]
// pub struct GUI2 {
//     pub windows: Arc<Mutex<Vec<Window2>>>,
//     glfw_window: Arc<Mutex<&'static c_void>>,
//     io: Arc<Mutex<&'static c_void>>,
// }

// impl GUI2 {
//     pub fn new(label: &str) -> Self {
//         let window_handle;
//         unsafe {
//             window_handle = init_gui("label\0".as_ptr());
//         }
//         GUI2 {
//             windows: Arc::new(Mutex::new(vec![])),
//             glfw_window: Arc::new(Mutex::new(window_handle.window)),
//             io: Arc::new(Mutex::new(window_handle.io)),
//         }
//     }

//     pub fn add_window(mut self, window: Window2) -> Self {
//         self.windows.lock().unwrap().push(window);
//         self
//     }

//     pub fn update(&mut self) {
//         unsafe { start_frame() }
//         for window in &mut *self.windows.lock().unwrap() {
//             window.update();
//         }
//         unsafe {
//             end_frame(
//                 *self.glfw_window.lock().unwrap(),
//                 *self.io.lock().unwrap(),
//                 ImGui_Vec4 {
//                     x: 1.0,
//                     y: 1.0,
//                     z: 1.0,
//                     w: 1.0,
//                 },
//             )
//         }
//     }

//     pub fn should_close(&self) -> bool {
//         unsafe {
//             if close_window(*self.glfw_window.lock().unwrap()) {
//                 return true;
//             }
//         }
//         false
//     }
// }

// #[derive(Clone)]
// pub struct Window2 {
//     pub buttons: Arc<Mutex<Vec<Button2>>>,
//     pub text: Arc<Mutex<Vec<Text2>>>,
//     pub text_input:Arc<Mutex<Vec<InputText2>>>,
// }

// impl Window2 {
//     pub fn new(label: &str) -> Self {
//         Window2 {
//             buttons: Arc::new(Mutex::new(vec![])),
//             text: Arc::new(Mutex::new(vec![])),
//             text_input: Arc::new(Mutex::new(vec![])),
//         }
//     }

//     pub fn add_button(self, button: Button2) -> Self {
//         self.buttons.lock().unwrap().push(button);
//         self
//     }

//     pub fn add_text(self, text: Text2) -> Self {
//         self.text.lock().unwrap().push(text);
//         self
//     }

//     pub fn add_input_text(self, text: InputText2) -> Self {
//         self.text_input.lock().unwrap().push(text);
//         self
//     }

//     pub fn update(&mut self) {
//         unsafe { ImGui_Begin("label\0".as_ptr(), &true, 0) }
//         for button in &mut *self.buttons.lock().unwrap() {
//             button.update()
//         }

//         for text in &mut *self.text.lock().unwrap() {
//             text.update()
//         }

//         for text_input in &mut *self.text_input.lock().unwrap() {
//             text_input.update()
//         }
//         unsafe {
//             ImGui_End();
//         }
//     }
// }

// #[derive(Debug, Clone)]
// pub struct Text2 {
//     pub label: Arc<Mutex<String>>,
// }

// impl Text2 {
//     pub fn new(label: &str) -> Self {
//         let mut label = String::from_str(label).unwrap();
//         if !label.ends_with("\0") {
//             label.push('\0');
//         }
//         Text2 {
//             label: Arc::new(Mutex::new(label)),
//         }
//     }
// }

// #[derive(Debug, Clone)]
// pub struct InputText2 {
//     pub label: Arc<Mutex<String>>,
//     pub value: Arc<Mutex<String>>,
//     // callback: Box<dyn Fn()>,
//     buffer_size: Arc<Mutex<i32>>,
//     pub flags: Arc<Mutex<i32>>,
// }

// impl InputText2 {
//     pub fn new(label: &str, size: i32) -> Self {
//         let mut string = String::new();
//         for _ in 0..size {
//             string.push('\0');
//         }

//         InputText2 {
//             label: Arc::new(Mutex::new(String::from_str(label).unwrap())),
//             value: Arc::new(Mutex::new(string)),
//             buffer_size: Arc::new(Mutex::new(size)),
//             flags: Arc::new(Mutex::new(0)),
//             // callback: Box::new(|| {}),
//         }
//     }

//     pub fn get_text(&self) -> String {
//         let null_terminator_position = self.value.lock().unwrap().find('\0').unwrap();

//         // if a truncation of a non ascii character occurred the string has a invalid memory layout past the intended end.
//         String::from_str(&self.value.lock().unwrap()[..null_terminator_position]).unwrap()
//     }
// }

// #[derive(Clone)]
// pub struct Button2 {
//     label: Arc<Mutex<String>>,
//     value: Arc<Mutex<bool>>,
//     pub callback: Arc<Mutex<dyn Fn() + Send>>,
// }

// impl Button2 {
//     pub fn new(label: &str) -> Self {
//         let mut label = String::from_str(label).unwrap();
//         if !label.ends_with("\0") {
//             label.push('\0');
//         }
//         Button2 {
//             value: Arc::new(Mutex::new(false)),
//             label: Arc::new(Mutex::new(label)),
//             callback: Arc::new(Mutex::new(|| {})),
//         }
//     }

//     pub fn callback(mut self, callback: impl Fn() + 'static + Send) -> Self {
//         self.callback = Arc::new(Mutex::new(callback));
//         self
//     }

//     pub fn call_callback(&mut self) {
//         let ets = &*self.value.lock().unwrap();
//         // let x = &*ets;
//         (self.callback.lock().unwrap())();
//     }
// }

//tmp
// impl Widget for Text2 {
//     fn update(&mut self) {
//         unsafe {
//             ImGui_Text(string_to_send_c_str(&*self.label.lock().unwrap()).as_ptr());
//         }
//     }
// }
//
//tmp
// impl Widget for InputText2 {
//     fn update(&mut self) {
//         unsafe {
//             ImGui_InputText(
//                 string_to_send_c_str(&*self.label.lock().unwrap()).as_ptr(),
//                 self.value.lock().unwrap().as_ptr().clone(),
//                 self.buffer_size.lock().unwrap().clone(),
//                 self.flags.lock().unwrap().clone(),
//             );
//         }
//     }
// }

// impl Widget for Button2 {
//     fn update(&mut self) {
//         let prev = self.value.lock().unwrap().clone();
//         unsafe {ImGui_Button("buttonlab\0".as_ptr(), &self.value.lock().unwrap());}
//         if prev != *self.value.lock().unwrap() {
//             self.call_callback();
//         }
//     }
// }

// macro_rules! impl_widget {
//     ($ty: ty, $(&self.$callback_val: ident,)? $fun: ident, $($(&self.$param: ident)? $(self.$param1: ident)? $(.$func: ident ())?), *) => {
//         paste::paste! {
//         impl Widget for $ty {
//                 fn update(&mut self) {
//                     $(let previous_val = self.$callback_val.lock().unwrap().clone();
//                         println!("val: {}", previous_val);
//                     )?
//                     // &*self.value.lock().unwrap()
//                     unsafe { [<$fun>](string_to_send_c_str(&self.label.lock().unwrap()).as_ptr(),
//                                         $($(&self.$param.lock().unwrap())?
//                                         $(self.$param1.lock().unwrap())?
//                                         $(.$func())?),
//                                     *); }

//                     $(
//                         println!("val: {}", *(*&self.$callback_val.lock().unwrap()));
//                         if previous_val != *(*&self.$callback_val.lock().unwrap()) {
//                             // (self.callback.lock().unwrap())();
//                             self.call_callback();
//                         }
//                     )?
//                 }
//             }
//         }
//     };
//     ($ty: ty, $fun: ident) => {
//         paste::paste! {
//         impl Widget for $ty {
//                 fn update(&mut self) {
//                     unsafe { [<$fun>](string_to_send_c_str(&self.label.lock().unwrap()).as_ptr()); }
//                 }
//             }
//         }
//     };
// }

// // impl_widget!(Button2, &self.value, ImGui_Button, &self.value);
// impl_widget!(Text2, ImGui_Text);
// impl_widget!(InputText2, ImGui_InputText, self.value.as_ptr(), self.buffer_size, self.flags);

// fn string_to_send_c_str(label: &String) -> String {
//     if label.ends_with("\0") {
//         label.clone()
//     } else {
//         let mut label = label.clone();
//         label.push('\0');
//         label
//     }
// }

// pub trait Widget {
//     fn update(&mut self);
// }
