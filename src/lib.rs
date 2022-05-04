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
        Arc::new(RwLock::new(self))
    }

    pub fn should_close(&self) -> bool {
        unsafe {
            if close_window(self.glfw_window.blocking_read().unwrap()) {
                return true;
            }
        }
        false
    }

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

pub type GuiHandle = Arc<RwLock<Gui>>;

pub trait AddWindow {
    fn window(self, window: Window) -> Self;
}

impl AddWindow for GuiHandle {
    fn window(mut self, window: Window) -> Self {
        self.blocking_write().windows.push(window);
        self
    }
}

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
                let gui = cp.blocking_read();

                let mut glfw_window = gui.glfw_window.blocking_write();
                *glfw_window = Some(window_handle.window);

                let mut io = gui.io.blocking_write();
                *io = Some(window_handle.io);
            }

            while !cp.blocking_read().should_close() {
                // let start_time = time::Instant::now();
                cp.blocking_read().update();
                tx.send(()).unwrap();
                // let time_delta = time::Instant::now() - start_time;
                // println!("{:?}", time_delta);
            }
        });
        let gui = self.blocking_read();
        let mut h = gui.thread_handle.blocking_write();
        *h = Some(handle);
        rx
    }

    fn is_running(&self) -> bool {
        let x = Arc::strong_count(self);
        x > 1 
        
    }
}

// impl Update for Gui {
//     fn update(&mut self) {
//         unsafe { start_frame() }
//         for window in &self.windows {
//             window.update();
//         }
//         let clear_color = ImGui_Vec4 {
//             x: 0.3,
//             y: 0.3,
//             z: 0.3,
//             w: 1.0,
//         };

//         unsafe {
//             end_frame(
//                 self.glfw_window.blocking_read().unwrap(),
//                 self.io.blocking_read().unwrap(),
//                 clear_color,
//             );
//         }
//     }
// }

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

    pub fn same_line<T>(mut self, widget: T) -> Self
    where Window: Add<T> {
        self.widgets.push(Arc::new(RwLock::new(SameLine::new(None, None))));
        self.add(widget)
    }

    fn update(&self) {
        unsafe { ImGui_Begin("windowlabel\0".as_ptr(), &true, 0) }
        for widget in &self.widgets {
            // widget.blocking_write().update();
            widget.blocking_write().update();
        }
        unsafe { ImGui_End() }
    }
}

pub trait Add<T> {
    fn add(self, widget: T) -> Self;    
}

impl Add<Checkbox> for Window {
    fn add(mut self, widget: Checkbox) -> Self {
        let widget = Arc::new(RwLock::new(widget));
        self.widgets.push(to_update(&widget));
        self.checkboxes.push(widget);
        self
    }
}

impl Add<InputText> for Window {
    fn add(mut self, widget: InputText) -> Self {
        let widget = Arc::new(RwLock::new(widget));
        self.widgets.push(to_update(&widget));
        self.text_input.push(widget);
        self
    }
}

impl Add<Text> for Window {
    fn add(mut self, widget: Text) -> Self {
        let widget = Arc::new(RwLock::new(widget));
        self.widgets.push(to_update(&widget));
        self.text.push(widget);
        self
    }
}

impl Add<Button> for Window {
    fn add(mut self, widget: Button) -> Self {
        let widget = Arc::new(RwLock::new(widget));
        self.widgets.push(to_update(&widget));
        self.buttons.push(widget);
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
        self.input_color.push(widget);    
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
            Widget::Checkbox(i) => self.checkboxes.get(i).expect("not enough checkboxes").blocking_read().value,
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
    fn update(&mut self);
}

// impl Update for Window {
//     fn update(&mut self) {
//         unsafe { ImGui_Begin("windowlabel\0".as_ptr(), &true, 0) }
//         for widget in &self.widgets {
//             widget.blocking_write().update();
//         }
//         unsafe { ImGui_End() }
//     }
// }

impl Update for Button {
    fn update(&mut self) {
        unsafe { ImGui_Button(self.label.as_ptr(), &self.value) }
        if self.value {
            (self.callback)();
        }
    }
}

impl Update for Text {
    fn update(&mut self) {
        unsafe { ImGui_Text(self.value.as_ptr()) }
    }
}

impl Update for Checkbox {
    fn update(&mut self) {
        unsafe { ImGui_Checkbox(self.label.as_ptr(), &self.value) }
    }
}

impl Update for InputText {
    fn update(&mut self) {
        unsafe { ImGui_InputText(self.label.as_ptr(), self.value.as_ptr(), self.buffersize, 0) }
    }
}

impl Update for InputColor {
    fn update(&mut self) {
        unsafe { ImGui_ColorEdit3(self.label.as_ptr(), &self.value) }
    }
}

impl Update for SameLine {
    fn update(&mut self) {
        unsafe { ImGui_SameLine(self.offset_from_start_x, self.spacing) }
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
    callback: Box<dyn Fn() + Send + Sync>,
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

    pub fn callback<T: 'static + Send + Sync + Fn()>(mut self, callback: T) -> Self {
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

pub struct SameLine {
    offset_from_start_x: f32,
    spacing: f32,
}

impl SameLine {
    pub fn new(offset_from_start_x: Option<f32>, spacing: Option<f32>) -> Self {
        let offset_from_start_x = if let Some(x) = offset_from_start_x {
            x
        } else {
            0.0
        };
        let spacing = if let Some(x) = spacing { x } else { -1.0 };

        SameLine {
            offset_from_start_x,
            spacing,
        }
    }
}

pub fn show_demo_window() {
    unsafe { backend::show_demo_window() }
}










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

// impl<'a> Drop for GUI<'a> {
//     fn drop(&mut self) {
//         unsafe {
//             destroy_gui(self.glfw_window);
//         }
//     }
// }

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////


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
