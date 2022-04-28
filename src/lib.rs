//! Example usage:
//!
//! ```
//! use rust_gui::*;
//! 
//! let button = Button::new();
//! let window = Window::new("example window".into());
//! build_window!(window, button);
//! let mut gui = GUI::new("example application".into());
//! gui.add_window(window.clone());
//! 
//! while !gui.should_close() {
//!     gui.update(None);
//! }
//! 
//! ```

use std::{cell::RefCell, ffi::{c_void, CString}, rc::Rc, sync::Mutex, str::FromStr};

use backend::*;
mod backend;

use rust_gui_macros::{Callback, ImGuiGlue};


/// This struct represents the handle to the Dear ImGui library. 
/// Use the Gui::new() function to create a context. You can add Windows to the gui by using the function gui.add_window().
/// After adding all windows, use the gui.update() function to render the gui.
pub struct GUI<'a> {
    pub windows: Vec<Rc<RefCell<Window<'a>>>>,
    glfw_window: &'static c_void,
    io: &'static c_void,
    should_close: bool,
}

#[macro_export]
//macro from webplatform
macro_rules! enclose {
    ( ($( $x:ident ),*) $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            $y
        }
    };
}

impl<'a> GUI<'a> {
    pub fn new(window_label: String) -> GUI<'a> {
        let mut window_label = window_label.clone();
        if window_label.len() == 0 {
            window_label.push(' ');
        }
        window_label.push('\0');
        unsafe {
            let window_handle = init_gui(window_label.as_ptr());
            GUI {
                windows: vec![],
                glfw_window: window_handle.window,
                io: window_handle.io,
                should_close: false,
            }
        }
    }

    pub fn add_window(&mut self, window: Rc<RefCell<Window<'a>>>) {
        self.windows.push(window);
    }

    pub fn update(&mut self, color: Option<ImGui_Vec4>) {
        unsafe {
            start_frame();
        }
        for i in 0..self.windows.len() {
            self.windows[i].borrow().render();
        }
        unsafe {
            let clear_color = if let Some(color) = color {
                color
            } else {
                ImGui_Vec4 {
                    x: 0.3,
                    y: 0.3,
                    z: 0.3,
                    w: 1.0,
                }
            };
            end_frame(self.glfw_window, self.io, clear_color.clone());
        }
    }

    pub fn should_close(&self) -> bool {
        unsafe {
            if close_window(self.glfw_window) {
                return true;
            }
        }
        self.should_close
    }
}

impl<'a> Drop for GUI<'a> {
    fn drop(&mut self) {
        unsafe {
            destroy_gui(self.glfw_window);
        }
    }
}

#[macro_export]
macro_rules! build_gui {
    ($a:expr, $b:expr) => {
        $a.add_window($b.clone());
    };

    ($a:expr, $b:expr, $c:expr) => {
        $a.add_window($b.clone());
        $a.add_window($c.clone());
    };

    ($a:expr, $b:expr, $($c:tt)*) => {
        $a.add_window($b.clone());
        build_window!($a,$($c)*)
    };
}

pub struct Window<'a> {
    pub items: Vec<&'a dyn ImGuiGlue>,
    pub show: bool,
    pub name: String,
    pub components: Vec<Rc<RefCell<dyn ImGuiGlue>>>,
    id: u32,
}

lazy_static::lazy_static! {
    static ref UNIQUE_WINDOW_ID: Mutex<u32> = Mutex::new(0u32);
}

impl<'a> Window<'a> {
    pub fn new(name: String) -> Rc<RefCell<Window<'a>>> {
        let mut id = UNIQUE_WINDOW_ID.lock().unwrap();
        let window = Rc::new(RefCell::new(Window {
            items: vec![],
            show: true,
            name,
            components: vec![],
            id: id.clone(),
        }));
        *id += 1;
        window
    }

    pub fn append<T: ImGuiGlue + 'static>(&mut self, comp: Rc<RefCell<T>>) {
        self.components.push(comp);
    }
}

#[macro_export]
macro_rules! build_window {
    ($a:expr, $b:expr) => {
        $a.borrow_mut().append($b.clone());
    };

    ($a:expr, $b:expr, $c:expr) => {
        $a.borrow_mut().append($b.clone());
        $a.borrow_mut().append($c.clone());
    };

    ($a:expr, $b:expr, $($c:tt)*) => {
        $a.borrow_mut().append($b.clone());
        build_window!($a,$($c)*)
    };
}

impl<'a> ImGuiGlue for Window<'a> {
    fn render(&self) {
        if self.show {
            let mut name = self.name.clone();
            // if name.is_empty() {
            //     name = " ".into();
            // }
            name.push_str("###");
            name.push_str(&self.id.to_string());
            name.push('\0');
            unsafe { ImGui_Begin(name.as_ptr(), &self.show, 0) }
            for item in &self.items {
                item.render();
            }
            for item in &self.components {
                item.borrow().render();
            }
            unsafe { ImGui_End() }
        }
    }
}

#[derive(Callback, ImGuiGlue)]
pub struct Checkbox {
    pub label: String,
    pub value: bool,
    pub callback: Box<dyn Fn()>,
}

impl Checkbox {
    pub fn new(label: String) -> Rc<RefCell<Checkbox>> {
        Rc::new(RefCell::new(Checkbox {
            label,
            value: false,
            callback: Box::new(|| {}),
        }))
    }
}

#[derive(ImGuiGlue)]
pub struct Text {
    pub label: String,
}

impl Text {
    pub fn new(label: String) -> Rc<RefCell<Text>> {
        Rc::new(RefCell::new(Text { label }))
    }
}

#[derive(Callback, ImGuiGlue)]
pub struct Button {
    pub label: String,
    pub value: bool,
    callback: Box<dyn Fn()>,
}

impl Button {
    pub fn new(label: String) -> Rc<RefCell<Button>> {
        Rc::new(RefCell::new(Button {
            label,
            value: false,
            callback: Box::new(|| {}),
        }))
    }
}

#[derive(ImGuiGlue)]
pub struct Color {
    label: String,
    pub value: ImGui_Vec4,
}

impl Color {
    pub fn new(label: String) -> Rc<RefCell<Color>> {
        let value = ImGui_Vec4 {
            x: 0.3,
            y: 0.3,
            z: 0.3,
            w: 1.0,
        };
        Rc::new(RefCell::new(Color { value, label }))
    }
}

#[derive(ImGuiGlue)]
pub struct SameLine {
    offset_from_start_x: f32,
    spacing: f32,
}

impl SameLine {
    pub fn new(offset_from_start_x: Option<f32>, spacing: Option<f32>) -> Rc<RefCell<Self>> {
        let offset_from_start_x = if let Some(x) = offset_from_start_x {
            x
        } else {
            0.0
        };
        let spacing = if let Some(x) = spacing { x } else { -1.0 };

        Rc::new(RefCell::new(SameLine {
            offset_from_start_x,
            spacing,
        }))
    }
}

pub fn show_demo_window() {
    unsafe { backend::show_demo_window() }
}

#[derive(Callback, ImGuiGlue)]
pub struct SliderInt {
    pub label: String,
    pub value: i32,
    callback: Box<dyn Fn()>,
    pub min: i32,
    pub max: i32,
}

impl SliderInt {
    pub fn new(label: String) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(SliderInt {
            label,
            value: 0,
            min: 0,
            max: 100,
            callback: Box::new(|| {}),
        }))
    }
}

#[derive(Callback, ImGuiGlue)]
pub struct SliderFloat {
    pub label: String,
    pub value: f32,
    callback: Box<dyn Fn()>,
    pub min: f32,
    pub max: f32,
}

impl SliderFloat {
    pub fn new(label: String) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(SliderFloat {
            label,
            value: 0.0,
            min: 0.0,
            max: 1.0,
            callback: Box::new(|| {}),
        }))
    }
}


#[derive(Callback)]
pub struct InputText {
    pub label: String,
    value: String,
    callback: Box<dyn Fn()>,
    buffer_size: i32,
    pub flags: i32,
}

impl ImGuiGlue for InputText {
    fn render(&self) {
        let prev = self.value.clone();

        let mut label = self.label.clone();
        if label.len() == 0 {
            label.push(' ');
        }
        label.push('\0');

        unsafe { ImGui_InputText(label.as_ptr(), self.value.as_ptr(), self.buffer_size, self.flags); }

        if prev != self.value {
            self.call_callback();
        }
    }
}

impl InputText {
    pub fn new(label: String, size: i32) -> Rc<RefCell<Self>> {
        let mut string = String::new();
        for _ in 0..size {
            string.push('\0');
        }
        
        Rc::new(RefCell::new( InputText {
            label,
            value: string,
            buffer_size: size,
            flags: 0,
            callback: Box::new(|| {}),
        }))
    }

    pub fn get_text(&self) -> &str {
        let null_terminator_position = self.value.find('\0').unwrap();
        
        // if a truncation of a non ascii character occurred the string has a invalid memory layout past the intended end.
        &self.value[..null_terminator_position]
    }
}

pub trait Callback {
    fn set_callback(&mut self, callback: impl Fn() + 'static);
    fn call_callback(&self);
}

//////////////////////////////////////////////////////////////
pub struct GUI2<T> {
    windows: Vec<Window2<T>>,
    glfw_window: &'static c_void,
    io: &'static c_void,
}

impl<T: Widget> GUI2<T> {
    pub fn new(label: &str) -> Self {
        let window_handle;
        unsafe { window_handle = init_gui("label\0".as_ptr()); }
        GUI2 { windows: vec![] , glfw_window: window_handle.window, io: window_handle.io}
    }

    pub fn add_window(mut self, window: Window2<T>) -> Self {
        self.windows.push(window);
        self
    }

    pub fn update(&mut self) {

        unsafe { start_frame() }
        for window in &mut self.windows {
            window.update();
        }
        unsafe { end_frame(self.glfw_window, self.io, ImGui_Vec4 { x: 1.0, y: 1.0, z: 1.0, w: 1.0 }) }
    }
}

pub struct Window2<T> {
    widgets: Vec<T>
}

impl<T: Widget> Window2<T> {
    pub fn new(label: &str) -> Self {
        Window2 { widgets: vec![] }
    }

    pub fn add(mut self, widget: T) -> Self {
        self.widgets.push(widget);
        self
    }

    pub fn update(&mut self) {
        unsafe { ImGui_Begin("label\0".as_ptr(), &true, 0) }
        for widget in &mut self.widgets {
            widget.update()
        }
        unsafe { ImGui_End(); }
    }

}
    

pub struct Text2 {
    label: String
}

impl Text2 {
    pub fn new() -> Self{
        Text2 { label: String::new() }
    }
}

pub struct InputText2 {
    label: String
}

impl InputText2 {
    pub fn new() -> Self {
        InputText2 { label: String::new() }
    }
}


pub struct Button2 {
    label: String,
    value: bool,
}

impl Button2 {
    pub fn new(label: &str) -> Self {
        Button2 { value: false, label: String::from_str("label\0").unwrap()}
    }
}


trait HasLabel {
    fn label(&self) -> &str;
}


trait HasCallback<T> {
    fn value(&self) -> T;
    fn callback(&self) -> fn() -> ();
}

trait Render {
    fn render(&mut self);
}

// impl Render for Button2 {
//     fn render(&mut self) {
//         let previous_val = self.val.clone();
//         unsafe { ImGui_Button(self.label.as_ptr(), &self.val) }
//     }
// } 

impl<T: Render + HasCallback<T> + Clone + PartialEq> Widget for T {
    fn update(&mut self) {
        let copy = self.value().clone();
        self.render();
        if copy != self.value() {
            self.callback()();
        }
    }
}

// impl<T: Render + ?HasCallback> Widget for T {
//     fn update(&mut self) {
//         self.render();
//     }
// }

#[macro_export]
macro_rules! fn_params {
    ($pat: tt $p1: ident) => {
        paste::paste! {
            [<$pat self.$p1>]
        }
    };

    ($p1: ident) => {
        paste::paste! {
            [<self.$p1>]
        }
    };

    ($pat1: tt $p1: ident, $($pat: tt $p: ident), *) => {
        fn_params!($pat1 $p1);, fn_params!($($pat $p)*);
    };
}

macro_rules! impl_widget {
    // ($ty: ty, $fun: ident, $pat: tt $p1: ident) => {
    //     paste::paste! {
    //     impl Widget for $ty {
    //             fn update(&mut self) {
    //                 unsafe { [<$fun>](string_to_send_c_str($pat self.$p1).as_ptr()); }
    //             }
    //         }
    //     }
    // };
    // ($ty: ty, $fun: ident, $($($ex: expr ;)? $pat: tt), *) => {
    //     paste::paste! {
    //     impl Widget for $ty {
    //             fn update(&mut self) {
    //                 unsafe { [<$fun>](string_to_send_c_str(&self.label).as_ptr(), $($pat)*); }
    //             }
    //         }
    //     }
    // };
    ($ty: ty, $fun: ident, $($(&self.$param: ident)? $(self.$param1: ident)?), *) => {
        paste::paste! {
        impl Widget for $ty {
                fn update(&mut self) {
                    unsafe { [<$fun>](string_to_send_c_str(&self.label).as_ptr(), $($(&self.$param)? $(self.$param1)?), *); }
                }
            }
        }
    };
    ($ty: ty, $fun: ident) => {
        paste::paste! {
        impl Widget for $ty {
                fn update(&mut self) {
                    unsafe { [<$fun>](string_to_send_c_str(&self.label).as_ptr()); }
                }
            }
        }
    };
}

// builder!(Text2, ImGui_Text,);
impl_widget!(Button2, ImGui_Button, &self.value);
impl_widget!(Text2, ImGui_Text);

fn string_to_send_c_str(label: &String) -> String {
    if label.ends_with("\0") {
        label.clone()
    } else {
        let mut label = label.clone();
        label.push('\0');
        label
    }
}

pub trait Widget {
    fn update(&mut self);
}
