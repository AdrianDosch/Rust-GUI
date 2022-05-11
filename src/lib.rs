mod backend;
use rust_gui_macros::*;

use backend::*;
use core::panic;
use std::{
    any::Any,
    ffi::c_void,
    str::FromStr,
    sync::{
        mpsc::{self, Receiver},
        Arc,
    },
    thread::{self, JoinHandle},
};
use tokio::sync::RwLock;

pub struct Gui {
    label: String,
    windows2: Vec<Window>,
    glfw_window: RwLock<Option<&'static c_void>>,
    io: RwLock<Option<&'static c_void>>,
    thread_handle: RwLock<Option<JoinHandle<()>>>,
    pub show_demo_window: RwLock<bool>,
}

impl Gui {
    pub fn new(label: &str) -> Gui {
        let mut label = String::from_str(label).unwrap();
        if !label.ends_with('\0') {
            label.push('\0');
        };

        Gui {
            label,
            windows2: vec![],
            glfw_window: RwLock::new(None),
            io: RwLock::new(None),
            thread_handle: RwLock::new(None),
            show_demo_window: RwLock::new(false),
        }
    }

    pub fn window(mut self, window: Window) -> Self {
        self.windows2.push(window);
        self
    }

    pub fn set<T: Set<U> + 'static, U>(&self, window_idx: usize, widget_idx: usize, value: U) {
        if let Some(window) = self.windows2.get(window_idx) {
            window.set_val::<T, U>(widget_idx, value);
        } else {
            panic!("not enough widgets");
        }
    }

    pub fn get<T: Get<U> + 'static, U>(&self, window_idx: usize, widget_idx: usize) -> U {
        if let Some(window) = self.windows2.get(window_idx) {
            window.get_val::<T, U>(widget_idx)
        } else {
            panic!("not enough widgets");
        }
    }

    pub fn get_widget<T: 'static + Clone>(&self, window_idx: usize, widget_idx: usize) -> T {
        if let Some(window) = self.windows2.get(window_idx) {
            window.get_widget(widget_idx)
        } else {
            panic!("not enough widgets");
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
        if *self.show_demo_window.blocking_read() {
            show_demo_window();
        }
        for window in &self.windows2 {
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
    fn start(&self) -> Receiver<()> {
        let cp = self.clone();
        let (tx, rx) = mpsc::sync_channel(1);
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
                match tx.try_send(()) {
                    Ok(_) => {}
                    Err(e) => match e {
                        mpsc::TrySendError::Full(_) => {}
                        mpsc::TrySendError::Disconnected(_) => panic!("sending error"),
                    },
                }
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

pub trait Update: Send + Sync {
    fn update(&self, gui: &Gui) -> bool;
    fn call_callback(&self, _gui: &Gui) {}
    fn set_callback<T: 'static + Send + Sync + Fn(&Gui)>(self, _callback: T) -> Self where Self: Sized {self}
    fn as_any(&self) -> &dyn Any;
}

pub trait Get<T> {
    fn get(&self) -> T;
}

pub trait Set<T> {
    fn set(&self, value: T);
}

pub trait Container2 {
    fn get_items(&self) -> &Vec<Arc<dyn Update>>;
    fn get_mut_items(&mut self) -> &mut Vec<Arc<dyn Update>>;
    fn get_val<T: 'static + Get<U>, U>(&self, idx: usize) -> U {
        let val = self
            .get_items()
            .iter()
            .filter_map(|x| x.as_any().downcast_ref::<T>())
            .nth(idx);

        if let Some(val) = val {
            val.get()
        } else {
            panic!("not found")
        }
    }

    fn set_val<T: 'static + Set<U>, U>(&self, idx: usize, value: U) {
        let val = self
            .get_items()
            .iter()
            .filter_map(|x| x.as_any().downcast_ref::<T>())
            .nth(idx);

        if let Some(val) = val {
            val.set(value);
        } else {
            panic!("not found");
        }
    }

    fn get_widget<T: 'static + Clone>(&self, widget_idx: usize) -> T{
        let widget = self.get_items()
        .iter()
        .filter_map(|x|{
            x.as_any()
            .downcast_ref::<T>()
        })
        .nth(widget_idx);
        
        if let Some(widget) = widget {
            widget.clone()
        } else {
            panic!("not enough widgets");
        }
    }

    fn add<T: Update + 'static>(mut self, widget: T) -> Self
    where
        Self: Sized,
    {
        self.get_mut_items().push(Arc::new(widget));
        self
    }

    fn same_line<T: Update + 'static>(self, widget: T) -> Self
    where
        Self: Sized,
    {
        self.add(SameLine::new(None, None)).add(widget)
    }
}

fn show_demo_window() {
    unsafe { backend::show_demo_window() }
}

#[derive(Clone)]
pub struct Window {
    label: String,
    widgets: Vec<Arc<dyn Update>>,
}

impl Window {
    pub fn new(label: &str) -> Self {
        let mut label = String::from_str(label).unwrap();
        if !label.ends_with('\0') {
            label.push('\0');
        };

        Window {
            label,
            widgets: vec![],
        }
    }
}

impl Update for Window {
    fn update(&self, gui: &Gui) -> bool {
        unsafe { ImGui_Begin(self.label.as_ptr(), &true, 0) }
        for widget in &self.widgets {
            if widget.update(gui) {
                widget.call_callback(gui);
            }
        }
        unsafe { ImGui_End() }
        false
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Container2 for Window {
    fn get_items(&self) -> &Vec<Arc<dyn Update>> {
        &self.widgets
    }
    fn get_mut_items(&mut self) -> &mut Vec<Arc<dyn Update>> {
        &mut self.widgets
    }
}

type Callback = Arc<RwLock<Box<dyn Fn(&Gui) + Send + Sync>>>;

#[derive(Clone)]
pub struct Button {
    label: Arc<RwLock<String>>,
    value: Arc<RwLock<bool>>,
    callback: Callback,
}

impl Button {
    pub fn new(label: &str) -> Self {
        let mut label = String::from_str(label).unwrap();
        if !label.ends_with('\0') {
            label.push('\0');
        };
        Button {
            label: Arc::new(RwLock::new(label)),
            value: Arc::new(RwLock::new(false)),
            callback: Arc::new(RwLock::new(Box::new(|_gui: &Gui| {}))),
        }
    }
}

impl Update for Button {
    fn update(&self, _: &Gui) -> bool {
        unsafe {
            ImGui_Button(
                self.label.blocking_write().as_ptr(),
                &self.value.blocking_write(),
            )
        }
        *self.value.blocking_read()
    }

    fn call_callback(&self, gui: &Gui) {
        (self.callback.blocking_read())(gui);
    }

    fn set_callback<T: 'static + Send + Sync + Fn(&Gui)>(mut self, callback: T) -> Self {
        self.callback = Arc::new(RwLock::new(Box::new(callback)));
        self
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Get<bool> for Button {
    fn get(&self) -> bool {
        *self.value.blocking_read()
    }
}

#[derive(Clone)]
pub struct Text {
    pub value: Arc<RwLock<String>>,
}

impl Text {
    pub fn new(label: &str) -> Self {
        let mut label = String::from_str(label).unwrap();
        if !label.ends_with('\0') {
            label.push('\0');
        };
        Text {
            value: Arc::new(RwLock::new(label)),
        }
    }
}

impl Set<String> for Text {
    fn set(&self, value: String) {
        *self.value.blocking_write() = value;
    }
}

impl Get<String> for Text {
    fn get(&self) -> String {
        self.value.blocking_read().clone()
    }
}

impl_Update!(Text, ImGui_Text(self.value.blocking_write().as_ptr()));

#[derive(Clone)]
pub struct Checkbox {
    label: Arc<RwLock<String>>,
    value: Arc<RwLock<bool>>,
    callback: Callback,
}

impl Checkbox {
    pub fn new(label: &str) -> Self {
        let mut label = String::from_str(label).unwrap();
        if !label.ends_with('\0') {
            label.push('\0');
        };
        Checkbox {
            label: Arc::new(RwLock::new(label)),
            value: Arc::new(RwLock::new(false)),
            callback: Arc::new(RwLock::new(Box::new(|_gui: &Gui| {}))),
        }
    }
}

impl Set<bool> for Checkbox {
    fn set(&self, value: bool) {
        *self.value.blocking_write() = value;
    }
}

impl Get<bool> for Checkbox {
    fn get(&self) -> bool {
        *self.value.blocking_read()
    }
}

impl_Update!(
    Checkbox,
    ImGui_Checkbox(
        self.label.blocking_write().as_ptr(),
        &self.value.blocking_write()
    ),
    callback,
    value
);

#[derive(Clone)]
pub struct InputText {
    label: Arc<RwLock<String>>,
    value: Arc<RwLock<String>>,
    buffer_size: Arc<RwLock<i32>>,
}

impl InputText {
    pub fn new(label: &str) -> Self {
        let mut label = String::from_str(label).unwrap();
        if !label.ends_with('\0') {
            label.push('\0');
        };

        let mut string = String::new();
        for _ in 0..255 {
            string.push('\0');
        }
        InputText {
            label: Arc::new(RwLock::new(label)),
            value: Arc::new(RwLock::new(string)),
            buffer_size: Arc::new(RwLock::new(255)),
        }
    }
}

impl Get<String> for InputText {
    fn get(&self) -> String {
        let null_terminator_position = self.value.blocking_read().find('\0').unwrap();

        // if a truncation of a non ascii character occurred the string has a invalid memory layout past the intended end.
        String::from(&self.value.blocking_read()[..null_terminator_position])
    }
}

impl_Update!(
    InputText,
    ImGui_InputText(
        self.label.blocking_write().as_ptr(),
        self.value.blocking_write().as_ptr(),
        *self.buffer_size.blocking_write(),
        0
    )
);

#[derive(Clone)]
pub struct InputColor {
    label: Arc<RwLock<String>>,
    value: Arc<RwLock<ImGui_Vec4>>,
}

impl InputColor {
    pub fn new(label: &str) -> InputColor {
        let mut label = String::from_str(label).unwrap();
        if !label.ends_with('\0') {
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

impl Get<Vec<f32>> for InputColor {
    fn get(&self) -> Vec<f32> {
        let color = *self.value.blocking_read();
        vec![color.x, color.y, color.z, color.w]
    }
}

impl_Update!(
    InputColor,
    ImGui_ColorEdit3(
        self.label.blocking_write().as_ptr(),
        &self.value.blocking_write()
    )
);

#[derive(Clone)]
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

impl_Update!(
    SameLine,
    ImGui_SameLine(
        *self.offset_from_start_x.blocking_read(),
        *self.spacing.blocking_read()
    )
);

#[derive(Clone)]
pub struct SliderInt {
    label: Arc<RwLock<String>>,
    value: Arc<RwLock<i32>>,
    min: Arc<RwLock<i32>>,
    max: Arc<RwLock<i32>>,
    callback: Callback,
}

impl SliderInt {
    pub fn new(label: &str) -> Self {
        let mut label = String::from_str(label).unwrap();
        if !label.ends_with('\0') {
            label.push('\0');
        };

        SliderInt {
            label: Arc::new(RwLock::new(label)),
            value: Arc::new(RwLock::new(0)),
            min: Arc::new(RwLock::new(0)),
            max: Arc::new(RwLock::new(100)),
            callback: Arc::new(RwLock::new(Box::new(|_gui: &Gui| {}))),
        }
    }
}

impl Get<i32> for SliderInt {
    fn get(&self) -> i32 {
        *self.value.blocking_read()
    }
}

impl Set<i32> for SliderInt {
    fn set(&self, value: i32) {
        *self.value.blocking_write() = value;
    }
}

impl_Update!(
    SliderInt,
    ImGui_SliderInt(
        self.label.blocking_write().as_ptr(),
        &self.value.blocking_write(),
        *self.min.blocking_read(),
        *self.max.blocking_read()
    ),
    callback,
    value
);

#[derive(Clone)]
pub struct SliderFloat {
    label: Arc<RwLock<String>>,
    value: Arc<RwLock<f32>>,
    min: Arc<RwLock<f32>>,
    max: Arc<RwLock<f32>>,
    callback: Callback,
}

impl SliderFloat {
    pub fn new(label: &str) -> Self {
        let mut label = String::from_str(label).unwrap();
        if !label.ends_with('\0') {
            label.push('\0');
        };

        SliderFloat {
            label: Arc::new(RwLock::new(label)),
            value: Arc::new(RwLock::new(0.0)),
            min: Arc::new(RwLock::new(0.0)),
            max: Arc::new(RwLock::new(100.0)),
            callback: Arc::new(RwLock::new(Box::new(|_gui: &Gui| {}))),
        }
    }
}

impl Get<f32> for SliderFloat {
    fn get(&self) -> f32 {
        *self.value.blocking_read()
    }
}

impl Set<f32> for SliderFloat {
    fn set(&self, value: f32) {
        *self.value.blocking_write() = value;
    }
}

impl_Update!(
    SliderFloat,
    ImGui_SliderFloat(
        self.label.blocking_write().as_ptr(),
        &self.value.blocking_write(),
        *self.min.blocking_read(),
        *self.max.blocking_read()
    ),
    callback,
    value
);

#[derive(Clone)]
pub struct TreeNode {
    flags: Arc<RwLock<i32>>,
    label: Arc<RwLock<String>>,
    items: Vec<Arc<dyn Update>>,
}

impl TreeNode {
    pub fn new(label: &str) -> Self {
        let mut label = String::from_str(label).unwrap();
        if !label.ends_with('\0') {
            label.push('\0');
        };

        TreeNode {
            flags: Arc::new(RwLock::new(0)),
            label: Arc::new(RwLock::new(label)),
            items: vec![],
        }
    }
}

impl Container2 for TreeNode {
    fn get_items(&self) -> &Vec<Arc<dyn Update>> {
        &self.items
    }

    fn get_mut_items(&mut self) -> &mut Vec<Arc<dyn Update>> {
        &mut self.items
    }
}

impl Update for TreeNode {
    fn update(&self, gui: &Gui) -> bool {
        if unsafe {
            ImGUI_TreeNodeEx(
                self.label.blocking_read().as_ptr(),
                *self.flags.blocking_read(),
            )
        } {
            for widget in &self.items {
                let call_callback = widget.update(gui);
                if call_callback {
                    widget.call_callback(gui);
                }
            }
            unsafe { ImGui__TreePop() }
        }
        false
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
