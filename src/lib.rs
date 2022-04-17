// mod rust_imgui;
use std::{cell::RefCell, ffi::c_void, rc::Rc};

use backend::*;
mod backend;

use rust_imgui_macros::Callback;

pub struct GUI<'a> {
    pub windows: Vec<Rc<RefCell<Window<'a>>>>,
    glfw_window: &'static c_void,
    io: &'static c_void,
    should_close: bool,
}

pub struct Window<'a> {
    pub items: Vec<&'a dyn ImgGuiGlue>,
    show: bool,
    pub name: String,
    pub components: Vec<Rc<RefCell<dyn ImgGuiGlue>>>,
}

impl<'a> Window<'a> {
    pub fn new(name: String) -> Rc<RefCell<Window<'a>>> {
        Rc::new(RefCell::new(Window {
            items: vec![],
            show: true,
            name,
            components: vec![],
        }))
    }

    pub fn append<T: ImgGuiGlue + 'static>(&mut self, comp: Rc<RefCell<T>>) {
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
    pub fn new() -> GUI<'a> {
        unsafe {
            let window_handle = init_gui();
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

    pub fn update(&mut self, color: Option<ImVec4>) {
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
                ImVec4 {
                    x: 0.3,
                    y: 0.3,
                    z: 0.3,
                    w: 1.0,
                }
            };
            end_frame(self.glfw_window, self.io, clear_color);
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

impl ImgGuiGlue for Checkbox {
    fn render(&self) {
        let mut label = self.label.clone();
        label.push('\0');
        unsafe {
            ImGui_Checkbox(label.as_ptr(), &self.value);
        }
        if self.value {
            self.call_callback();
        }
    }
}

impl<'a> ImgGuiGlue for Window<'a> {
    fn render(&self) {
        if self.show {
            let mut name = self.name.clone();
            if name.is_empty() {
                name = " ".into();
            }
            name.push('\0');
            unsafe { ImGui_Begin(name.as_ptr(), &self.show) }
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

#[derive(Callback)]
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

pub struct Text {
    pub text: String,
}

impl Text {
    pub fn new(text: String) -> Rc<RefCell<Text>> {
        Rc::new(RefCell::new(Text { text }))
    }
}

impl ImgGuiGlue for Text {
    fn render(&self) {
        let mut text = self.text.clone();
        text.push('\0');
        unsafe {
            ImGui_Text(text.as_ptr());
        }
    }
}

#[derive(Callback)]
pub struct Button {
    pub text: String,
    pub value: bool,
    callback: Box<dyn Fn()>,
}

impl Button {
    pub fn new(text: String) -> Rc<RefCell<Button>> {
        Rc::new(RefCell::new(Button {
            text,
            value: false,
            callback: Box::new(||{}),
        }))
    }
}

impl ImgGuiGlue for Button {
    fn render(&self) {
        let mut text = self.text.clone();
        text.push('\0');
        unsafe {
            ImGui_Button(text.as_ptr(), &self.value);
        }
        if self.value {
            self.call_callback();
        }
    }
}

pub struct Color {
    pub col: ImVec4,
    label: String,
}

impl Color {
    pub fn new(label: String) -> Rc<RefCell<Color>> {
        let col = ImVec4 {
            x: 0.3,
            y: 0.3,
            z: 0.3,
            w: 1.0,
        };
        Rc::new(RefCell::new(Color { col, label }))
    }
}

impl ImgGuiGlue for Color {
    fn render(&self) {
        let mut label = self.label.clone();
        label.push('\0');
        unsafe { ImGui_ColorEdit3(label.as_ptr(), &self.col) }
    }
}

pub fn show_demo_window() {
    unsafe { backend::show_demo_window() }
}

pub trait Callback {
    fn set_callback(&mut self, callback: impl Fn() + 'static);
    fn call_callback(&self);
}
