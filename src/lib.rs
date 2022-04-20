// mod rust_imgui;
use std::{cell::RefCell, ffi::c_void, rc::Rc};

use backend::*;
mod backend;

use rust_imgui_macros::{Callback, ImgGuiGlue};

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

#[derive(Callback, ImgGuiGlue)]
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

#[derive(ImgGuiGlue)]
pub struct Text {
    pub label: String,
}

impl Text {
    pub fn new(label: String) -> Rc<RefCell<Text>> {
        Rc::new(RefCell::new(Text { label }))
    }
}

#[derive(Callback, ImgGuiGlue)]
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
            callback: Box::new(||{}),
        }))
    }
}

#[derive(ImgGuiGlue)]
pub struct Color {
    label: String,
    pub value: ImVec4,
}

impl Color {
    pub fn new(label: String) -> Rc<RefCell<Color>> {
        let value = ImVec4 {
            x: 0.3,
            y: 0.3,
            z: 0.3,
            w: 1.0,
        };
        Rc::new(RefCell::new(Color { value, label }))
    }
}

#[derive(ImgGuiGlue)]
pub struct SameLine {
    offset_from_start_x: f32,
    spaceing: f32
}

impl SameLine {
    pub fn new(offset_from_start_x: Option<f32>, spaceing: Option<f32>) ->  Rc<RefCell<Self>> {
        let offset_from_start_x = if let Some(x) = offset_from_start_x {x} else {0.0};
        let spaceing = if let Some(x) = spaceing {x} else {-1.0};
        
        Rc::new(RefCell::new(SameLine { offset_from_start_x, spaceing }))
    }
}

pub fn show_demo_window() {
    unsafe { backend::show_demo_window() }
}

#[derive(Callback, ImgGuiGlue)]
pub struct SliderInt {
    pub label: String,
    pub value: i32,
    callback: Box<dyn Fn()>,
    pub min: i32,
    pub max: i32
}

impl SliderInt {
    pub fn new(label: String) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(SliderInt { label, value: 0, min: 0, max: 100, callback: Box::new(||{}) }))
    }
}

#[derive(Callback, ImgGuiGlue)]
pub struct SliderFloat {
    pub label: String,
    pub value: f32,
    callback: Box<dyn Fn()>,
    pub min: f32,
    pub max: f32
}

impl SliderFloat {
    pub fn new(label: String) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(SliderFloat { label, value: 0.0, min: 0.0, max: 1.0, callback: Box::new(||{}) }))
    }
}
pub trait Callback {
    fn set_callback(&mut self, callback: impl Fn() + 'static);
    fn call_callback(&self);
}
