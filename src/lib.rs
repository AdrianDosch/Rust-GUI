mod backend;

use backend::*;
use std::{
    ffi::c_void,
    thread::{self, JoinHandle},
    time::{self, Duration}, sync::{Arc, mpsc::{self, Receiver}}, str::FromStr,
};
use tokio::sync::RwLock;

pub struct Gui {
    label: String,
    windows: Vec<Window>,
    glfw_window: RwLock<Option<&'static c_void>>,
    io: RwLock<Option<&'static c_void>>,
    should_close: RwLock<bool>,
    thread_handle: RwLock<Option<JoinHandle<()>>>,
}

impl Gui {
    pub fn new(label: &str) -> Gui {
        let mut label = String::from_str(label).unwrap();
        if !label.ends_with("\0"){
            label.push('\0');
        };
        
        Gui {
            label,
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

    fn should_close(&self) -> bool {
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
            window.update(self);
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
                let window_handle = init_gui(cp.label.as_ptr());
                             
                let mut glfw_window = cp.glfw_window.blocking_write();
                *glfw_window = Some(window_handle.window);

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
    label: String,
    buttons: Vec<Arc<Button>>,
    text: Vec<Arc<Text>>,
    checkboxes: Vec<Arc<Checkbox>>,
    text_input: Vec<Arc<InputText>>,
    input_color: Vec<Arc<InputColor>>,
    widgets: Vec<Arc<dyn Update + Send + Sync>>,
}

impl Window {
    pub fn new(label: &str) -> Self {
        let mut label = String::from_str(label).unwrap();
        if !label.ends_with("\0"){
            label.push('\0');
        };

        Window {
            label,
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
        self.widgets.push(Arc::new(SameLine::new(None, None)));
        self.add(widget)
    }

    fn update(&self, gui: &Gui) {
        unsafe { ImGui_Begin(self.label.as_ptr(), &true, 0) }
        for widget in &self.widgets {
            // widget.blocking_write().update();
            let x = widget.update();
            if x {
                widget.call_callback(gui);
            }
        }
        unsafe { ImGui_End() }
    }
}

pub trait Add<T> {
    fn add(self, widget: T) -> Self;    
}

impl Add<Checkbox> for Window {
    fn add(mut self, widget: Checkbox) -> Self {
        let widget = Arc::new(widget);
        self.widgets.push(to_update(&widget));
        self.checkboxes.push(widget);
        self
    }
}

impl Add<InputText> for Window {
    fn add(mut self, widget: InputText) -> Self {
        let widget = Arc::new(widget);
        self.widgets.push(to_update(&widget));
        self.text_input.push(widget);
        self
    }
}

impl Add<Text> for Window {
    fn add(mut self, widget: Text) -> Self {
        let widget = Arc::new(widget);
        self.widgets.push(to_update(&widget));
        self.text.push(widget);
        self
    }
}

impl Add<Button> for Window {
    fn add(mut self, widget: Button) -> Self {
        let widget = Arc::new(widget);
        self.widgets.push(to_update(&widget));
        self.buttons.push(widget);
        self
    }
}


fn to_update<T: Update + 'static + Send + Sync>(test: &Arc<T>) -> Arc<dyn Update + 'static + Send + Sync> {
    let x = test.clone();
    x
}

impl Add<InputColor> for Window {
    fn add(mut self, widget: InputColor) -> Self {
        let widget = Arc::new(widget);   
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
                .set(value),
            Widget::Checkbox(i) => self
                .checkboxes
                .get(i)
                .expect("msg")
                .set(value),
            _ => unreachable!("set widget: {:?} not implemented", widget),
        }
    }

    fn get(&self, widget: Widget) -> bool {
        match widget {
            Widget::Button(i) => {
                *self.buttons
                    .get(i)
                    .expect("there aren't enough buttons")
                    .value.blocking_read()
            }
            Widget::Checkbox(i) => *self.checkboxes.get(i).expect("not enough checkboxes").value.blocking_read(),
            _ => unreachable!("set widget: {:?} not implemented", widget),
        }
    }
}

impl AccessWidget<String> for Window {
    fn set(&self, widget: Widget, value: String) {
        let mut value = value;
        if !value.ends_with("\0"){
            value.push('\0');
        };
        match widget {
            Widget::Text(i) => self
                .text
                .get(i)
                .expect("there aren't enough text widgets")
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
                .value
                .blocking_read()
                .clone(),
            Widget::InputText(i) => self
                .text_input
                .get(i)
                .expect("not enough input text")
                .get_text()
                .to_string(),
            _ => unreachable!("set widget: {:?} not implemented", widget),
        }
    }
}

pub trait Update {
    fn update(&self) -> bool;
    fn call_callback(&self, gui: &Gui) {

    }
}


impl Update for Button {
    fn update(&self) -> bool{
        unsafe { ImGui_Button(self.label.blocking_write().as_ptr(), &self.value.blocking_write()) }
        *self.value.blocking_read()
    }

    fn call_callback(&self, gui: &Gui) {
        (self.callback.blocking_read())(gui);
    }
}

impl Update for Text {
    fn update(&self) -> bool{
        unsafe { ImGui_Text(self.value.blocking_write().as_ptr()) }
        false
    }
}

impl Update for Checkbox {
    fn update(&self) -> bool{
        unsafe { ImGui_Checkbox(self.label.blocking_write().as_ptr(), &self.value.blocking_write()) }
        false
    }
}

impl Update for InputText {
    fn update(&self) -> bool{
        unsafe { ImGui_InputText(self.label.blocking_write().as_ptr(), self.value.blocking_write().as_ptr(), *self.buffersize.blocking_write(), 0) }
        false
    }
}

impl Update for InputColor {
    fn update(&self) -> bool{
        unsafe { ImGui_ColorEdit3(self.label.blocking_write().as_ptr(), &self.value.blocking_write()) }
        false
    }
}

impl Update for SameLine {
    fn update(&self) -> bool{
        unsafe { ImGui_SameLine(*self.offset_from_start_x.blocking_read(), *self.spacing.blocking_read()) }
        false
    }
}

pub trait Set<T> {
    fn set(&self, value: T);
}

impl Set<bool> for Button {
    fn set(&self, value: bool) {
        *self.value.blocking_write() = value;
    }
}

impl Set<String> for Text {
    fn set(&self, value: String) {
        *self.value.blocking_write() = value;
    }
}

impl Set<bool> for Checkbox {
    fn set(&self, value: bool) {
        *self.value.blocking_write() = value;
    }
}

impl Set<Vec<f32>> for InputColor {
    fn set(&self, value: Vec<f32>) {
        *self.value.blocking_write() = ImGui_Vec4 {x: value[0], y: value[1], z: value[2], w: value[3]};
    }
}

pub struct Button {
    label: Arc<RwLock<String>>,
    value: Arc<RwLock<bool>>,
    callback: Arc<RwLock<Box<dyn Fn(&Gui) + Send + Sync>>>,
}

impl Button {
    pub fn new(label: &str) -> Self {
        let mut label = String::from_str(label).unwrap();
        if !label.ends_with("\0"){
            label.push('\0');
        };
        Button {
            label: Arc::new(RwLock::new(label)),
            value: Arc::new(RwLock::new(false)),
            callback: Arc::new(RwLock::new(Box::new(|gui: &Gui| {}))),
        }
    }

    pub fn callback<T: 'static + Send + Sync + Fn(&Gui)>(mut self, callback: T) -> Self {
        self.callback = Arc::new(RwLock::new(Box::new(callback)));
        self
    }
}


pub struct Text {
    pub value: Arc<RwLock<String>>,
}

impl Text {
    pub fn new(label: &str, ) -> Self {
        let mut label = String::from_str(label).unwrap();
        if !label.ends_with("\0"){
            label.push('\0');
        };
        Text {
            value: Arc::new(RwLock::new(label)),
        }
    }
}

pub struct Checkbox {
    label: Arc<RwLock<String>>,
    pub value: Arc<RwLock<bool>>,
}

impl Checkbox {
    pub fn new(label: &str) -> Self {
        let mut label = String::from_str(label).unwrap();
        if !label.ends_with("\0"){
            label.push('\0');
        };
        Checkbox { label: Arc::new(RwLock::new(label)), value: Arc::new(RwLock::new(false)) }
    }
}

pub struct InputText {
    label: Arc<RwLock<String>>,
    value: Arc<RwLock<String>>,
    buffersize: Arc<RwLock<i32>>,
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
            label: Arc::new(RwLock::new(label)),
            value: Arc::new(RwLock::new(string)),
            buffersize: Arc::new(RwLock::new(255)),
        }
    }

    pub fn get_text(&self) -> String {
        let null_terminator_position = self.value.blocking_read().find('\0').unwrap();

        // if a truncation of a non ascii character occurred the string has a invalid memory layout past the intended end.
        String::from(&self.value.blocking_read()[..null_terminator_position])
    }
}

pub struct InputColor {
    label: Arc<RwLock<String>>,
    value: Arc<RwLock<ImGui_Vec4>>,
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
        InputColor { 
            label: Arc::new(RwLock::new(label)),
            value: Arc::new(RwLock::new(value)),
        }
    }
}

pub struct SameLine {
    offset_from_start_x: Arc<RwLock<f32>>,
    spacing: Arc<RwLock<f32>>,
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
            offset_from_start_x: Arc::new(RwLock::new(offset_from_start_x)),
            spacing: Arc::new(RwLock::new(spacing)),
        }
    }
}

pub fn show_demo_window() {
    unsafe { backend::show_demo_window() }
}





//future implementations:

// impl SliderInt {
//     pub fn new(label: String) -> Rc<RwLock<Self>> {
//         Rc::new(RwLock::new(SliderInt {
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
//     pub fn new(label: String) -> Rc<RwLock<Self>> {
//         Rc::new(RwLock::new(SliderFloat {
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
