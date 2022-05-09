mod backend;

use backend::*;
use std::{
    ffi::c_void,
    str::FromStr,
    sync::{
        mpsc::{self, Receiver},
        Arc,
    },
    thread::{self, JoinHandle},
};
use tokio::sync::RwLock;

macro_rules! callback {
    ($name: ident) => {
        pub fn callback<T: 'static + Send + Sync + Fn(&Gui)>(mut self, callback: T) -> Self {
            self.$name = Arc::new(RwLock::new(Box::new(callback)));
            self
        }
    };
}

pub struct Gui {
    label: String,
    windows: Vec<Window>,
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
            windows: vec![],
            glfw_window: RwLock::new(None),
            io: RwLock::new(None),
            thread_handle: RwLock::new(None),
            show_demo_window: RwLock::new(false),
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
            Ok(window.get(widget))
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
        if *self.show_demo_window.blocking_read() {
            show_demo_window();
        }
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

// impl Drop for Gui {
//     fn drop(&mut self) {
//         unsafe {
//             if let Some(window_handle) = *self.glfw_window.blocking_write(){
//                 destroy_gui(&window_handle);
//             }
//         }
//     }
// }

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

#[derive(Debug, Clone)]
pub struct WidgetNotFoundError;

#[derive(Debug)]
pub enum Widget {
    Button(usize),
    Text(usize),
    Checkbox(usize),
    InputText(usize),
    InputColor(usize),
    SliderInt(usize),
    SliderFloat(usize),
    TreeNode(usize),
}

enum Item {
    Button(Button),
    Text(Text),
    Checkbox(Checkbox),
    InputText(InputText),
    InputColor(InputColor),
    SliderInt(SliderInt),
    SliderFloat(SliderFloat),
    TreeNode(TreeNode)
}

struct Container {
    items: Vec<Item>
}

impl Contain for Container {
    fn get_items(&self) -> &Vec<Item>{
        &self.items
    }
}

trait Contain {
    fn get_items(&self) -> &Vec<Item>;
}

trait Setter<T> {
    fn set(&self, value: T);
}

trait Getter<T> {
    fn get(&self) -> T;
}

impl Setter<bool> for Item {
    fn set(&self, value: bool) {
        match self {
            Item::Button(b) => *b.value.blocking_write() = value,
            Item::Checkbox(c) => *c.value.blocking_write() = value,
            _ => unimplemented!()
        }
    }
}

impl Setter<String> for Item {
    fn set(&self, value: String) {
        match self {
            Item::Text(_) => todo!(),
            Item::InputText(_) => todo!(),
            _ => unimplemented!()
        }
    }
}

impl Getter<bool> for Item {
    fn get(&self) -> bool {
        match self {
            Item::Button(b) => *b.value.blocking_read(),
            Item::Checkbox(c) => *c.value.blocking_read(),
            _ => unimplemented!()
        }
    }
}

impl Getter<String> for Item {
    fn get(&self) -> String {
        match self {
            Item::Text(t) => t.value.blocking_read().clone(),
            Item::InputText(i) => i.value.blocking_read().clone(),
            _ => unimplemented!()
        }
    }
}


trait WidgetHdl {
    fn set<T>(&self, widget: Widget, value: T)
    where Item: Setter<T>;
    fn get<T>(&self, widget: Widget) -> T
    where Item: Getter<T>;
}

fn is_same(item: &Item, widget: &Widget) -> bool {
    match item {
        Item::Button(_) => match widget {Widget::Button(_) => true, _ => false},
        Item::Checkbox(_) => match widget {Widget::Checkbox(_) => true, _ => false},
        Item::Text(_) =>  match widget {Widget::InputText(_) => true, _ => false},
        Item::InputText(_) =>  match widget {Widget::Checkbox(_) => true, _ => false},
        Item::InputColor(_) =>  match widget {Widget::InputColor(_) => true, _ => false},
        Item::SliderInt(_) =>  match widget {Widget::SliderInt(_) => true, _ => false},
        Item::SliderFloat(_) =>  match widget {Widget::SliderFloat(_) => true, _ => false},
        Item::TreeNode(_) =>  match widget {Widget::TreeNode(_) => true, _ => false},
    }
}

trait Index {
    fn idx(&self) -> usize;
}

impl Index for Widget {
    fn idx(&self) -> usize{
        match self {
            Widget::Button(i) |
            Widget::Checkbox(i) |
            Widget::InputColor(i) |
            Widget::InputText(i) |
            Widget::SliderFloat(i) |
            Widget::SliderInt(i) |
            Widget::Text(i) | 
            Widget::TreeNode(i) => *i
        }
    }
}

impl WidgetHdl for dyn Contain {
    fn set<T>(&self, widget: Widget, value: T) 
    where Item: Setter<T> {
        let count = widget.idx();
        self.get_items().iter()
        .filter(|x| {
            is_same(x, &widget)
        })
        .nth(count)
        .and_then(|x| {
            x.set(value);
            Some(())
        });     
    }

    fn get<T>(&self, widget: Widget) -> T 
    where Item: Getter<T> {
        let count = widget.idx();
        let item = self.get_items().iter()
        .filter(|x| {
            is_same(x, &widget)
        })
        .nth(count)
        .and_then(|x|{
            Some(x.get())
        });

        if let Some(item) = item {
            return item;
        } else {
            panic!("not enough widgets");
        }
    }
}

impl AccessWidget<bool> for Container {
    fn set(&self, widget: Widget, value: bool) {
        let count = match widget {
            Widget::Button(i) | Widget::Checkbox(i) => i,
            _ => unimplemented!()
        };
        self.items.iter().filter(|x| {
            match x {
                Item::Button(_) => match widget {Widget::Button(_) => true, _ => false},
                Item::Checkbox(_) => match widget {Widget::Checkbox(_) => true, _ => false},
                _ => unimplemented!()
            }
        })
        .nth(count)
        .and_then(|x| {
            match x {
                Item::Button(b) => Some(*b.value.blocking_write() = value),
                Item::Checkbox(c) => Some(*c.value.blocking_write() = value),
                _ => None
            }
        });
    }

    fn get(&self, widget: Widget) -> bool {
        todo!()
    }
}

pub struct Window {
    label: String,
    buttons: Vec<Arc<Button>>,
    text: Vec<Arc<Text>>,
    checkboxes: Vec<Arc<Checkbox>>,
    text_input: Vec<Arc<InputText>>,
    input_color: Vec<Arc<InputColor>>,
    slider_int: Vec<Arc<SliderInt>>,
    slider_float: Vec<Arc<SliderFloat>>,
    tree_nodes: Vec<Arc<TreeNode>>,
    widgets: Vec<Arc<dyn Update + Send + Sync>>,
}

impl Window {
    pub fn new(label: &str) -> Self {
        let mut label = String::from_str(label).unwrap();
        if !label.ends_with('\0') {
            label.push('\0');
        };

        Window {
            label,
            buttons: vec![],
            text: vec![],
            checkboxes: vec![],
            text_input: vec![],
            input_color: vec![],
            slider_int: vec![],
            slider_float: vec![],
            tree_nodes: vec![],
            widgets: vec![],
        }
    }

    pub fn same_line<T>(mut self, widget: T) -> Self
    where
        Window: Add<T>,
    {
        self.widgets.push(Arc::new(SameLine::new(None, None)));
        self.add(widget)
    }

    fn update(&self, gui: &Gui) {
        unsafe { ImGui_Begin(self.label.as_ptr(), &true, 0) }
        for widget in &self.widgets {
            let x = widget.update(gui);
            if x {
                widget.call_callback(gui);
            }
        }
        unsafe { ImGui_End() }
    }
}

pub trait AccessWidget<T> {
    fn set(&self, widget: Widget, value: T);
    fn get(&self, widget: Widget) -> T;
}

impl AccessWidget<bool> for Window {
    fn set(&self, widget: Widget, value: bool) {
        match widget {
            Widget::Button(i) => self
                .buttons
                .get(i)
                .expect("there aren't enough buttons")
                .set(value),
            Widget::Checkbox(i) => self.checkboxes.get(i).expect("msg").set(value),
            _ => unreachable!("set widget: {:?} not implemented", widget),
        }
    }

    fn get(&self, widget: Widget) -> bool {
        match widget {
            Widget::Button(i) => *self
                .buttons
                .get(i)
                .expect("there aren't enough buttons")
                .value
                .blocking_read(),
            Widget::Checkbox(i) => *self
                .checkboxes
                .get(i)
                .expect("not enough checkboxes")
                .value
                .blocking_read(),
            _ => unreachable!("set widget: {:?} not implemented", widget),
        }
    }
}

impl AccessWidget<String> for Window {
    fn set(&self, widget: Widget, value: String) {
        let mut value = value;
        if !value.ends_with('\0') {
            value.push('\0');
        };
        match widget {
            Widget::Text(i) => self
                .text
                .get(i)
                .expect("there aren't enough text widgets")
                .set(value),
            Widget::InputText(i) => {
                // if *self
                //     .text_input
                //     .get(i)
                //     .expect("tere aren't enough text inputs")
                //     .buffer_size
                //     .blocking_read()
                //     <
                //     value.capacity() {
                //         panic!("the string input is to large");
                //     }
                *self
                .text_input
                .get(i)
                .expect("there aren't enough text inputs")
                .value
                .blocking_write() = value;
            }
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
                .get_text(),
            _ => unreachable!("set widget: {:?} not implemented", widget),
        }
    }
}

pub trait Update {
    fn update(&self, gui: &Gui) -> bool;
    fn call_callback(&self, _gui: &Gui) {}
}

pub trait Add<T> {
    fn add(self, widget: T) -> Self;
}

pub trait Set<T> {
    fn set(&self, value: T);
}

fn to_update<T: Update + 'static + Send + Sync>(
    test: &Arc<T>,
) -> Arc<dyn Update + 'static + Send + Sync> {
    test.clone() as _
}

fn show_demo_window() {
    unsafe { backend::show_demo_window() }
}

type Callback = Arc<RwLock<Box<dyn Fn(&Gui) + Send + Sync>>>;

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

    pub fn callback<T: 'static + Send + Sync + Fn(&Gui)>(mut self, callback: T) -> Self {
        self.callback = Arc::new(RwLock::new(Box::new(callback)));
        self
    }
}

impl Update for Button {
    fn update(&self, _: &Gui) -> bool {
        unsafe {ImGui_Button(
            self.label.blocking_write().as_ptr(),
            &self.value.blocking_write())
        }
        *self.value.blocking_read()
    }

    fn call_callback(&self, gui: &Gui) {
        (self.callback.blocking_read())(gui);
    }
}

impl_Add!(Button, buttons);
impl_Set!(Button, bool, value);

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

impl_Update!(Text, ImGui_Text(self.value.blocking_write().as_ptr()));
impl_Add!(Text, text);
impl_Set!(Text, String, value);

pub struct Checkbox {
    label: Arc<RwLock<String>>,
    pub value: Arc<RwLock<bool>>,
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

    callback!(callback);
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
impl_Add!(Checkbox, checkboxes);
impl_Set!(Checkbox, bool, value);

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

    pub fn get_text(&self) -> String {
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
impl_Add!(InputText, text_input);
impl_Set!(InputText, String, value);

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

impl Set<Vec<f32>> for InputColor {
    fn set(&self, value: Vec<f32>) {
        *self.value.blocking_write() = ImGui_Vec4 {
            x: value[0],
            y: value[1],
            z: value[2],
            w: value[3],
        };
    }
}

impl_Update!(
    InputColor,
    ImGui_ColorEdit3(
        self.label.blocking_write().as_ptr(),
        &self.value.blocking_write()
    )
);
impl_Add!(InputColor, input_color);

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

    callback!(callback);
}

use rust_gui_macros::*;

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
impl_Add!(SliderInt, slider_int);
impl_Set!(SliderInt, i32, value);

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

    callback!(callback);
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
impl_Add!(SliderFloat, slider_float);
impl_Set!(SliderFloat, f32, value);

pub struct TreeNode {
    flags: Arc<RwLock<i32>>,
    label: Arc<RwLock<String>>,
    buttons: Vec<Arc<Button>>,
    text: Vec<Arc<Text>>,
    checkboxes: Vec<Arc<Checkbox>>,
    text_input: Vec<Arc<InputText>>,
    input_color: Vec<Arc<InputColor>>,
    slider_int: Vec<Arc<SliderInt>>,
    slider_float: Vec<Arc<SliderFloat>>,
    tree_nodes: Vec<Arc<TreeNode>>,
    widgets: Vec<Arc<dyn Update + Send + Sync>>,
}

impl TreeNode {
    pub fn new(label: &str) -> Self{
        let mut label = String::from_str(label).unwrap();
        if !label.ends_with('\0') {
            label.push('\0');
        };

        TreeNode {
            flags: Arc::new(RwLock::new(0)),
            label: Arc::new(RwLock::new(label)),
            buttons: vec![],
            text: vec![],
            checkboxes: vec![],
            text_input: vec![],
            input_color: vec![],
            slider_int: vec![],
            slider_float: vec![],
            tree_nodes: vec![],
            widgets: vec![],
        }
    }

    pub fn same_line<T>(mut self, widget: T) -> Self
    where
        TreeNode: Add<T>,
    {
        self.widgets.push(Arc::new(SameLine::new(None, None)));
        self.add(widget)
    }
}

impl Update for TreeNode {
    fn update(&self, gui: &Gui) -> bool {
       if unsafe { ImGUI_TreeNodeEx(self.label.blocking_read().as_ptr(), *self.flags.blocking_read()) }{
           for widget in &self.widgets {
                let call_callback = widget.update(gui);
                if call_callback {
                    widget.call_callback(gui);
                }
           }
           unsafe { ImGui__TreePop() }
       }
        false
    }
}

// impl Contain for TreeNode {
//     fn get_items(&self) -> &Vec<Item> {
        
//     }
// }

impl_Add!(TreeNode, tree_nodes);
