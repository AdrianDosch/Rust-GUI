mod imgui;
use std::ffi::{self, c_void};

use imgui::*;

pub struct GUI<T: ImgGuiGlue> {
    pub windows: Vec<Window<T>>,
    glfw_window: &'static c_void,
    should_close: bool,
}

pub struct Window<T: ImgGuiGlue> {
    pub items: Vec<T>
}

impl<T: ImgGuiGlue> GUI<T> {
    pub fn new() -> GUI<T> {
        unsafe {
            let window_handle = init_gui1();
            GUI { windows: vec![], glfw_window: window_handle.window, should_close: false}
        }
    }

    pub fn add_window(&mut self, window: Window<T>) {
        self.windows.push(window);
    }

    pub fn update(&mut self) {
        unsafe {start_frame1();}
        for widow in &mut self.windows {
            widow.render();
        }
        unsafe {end_frame1(self.glfw_window, ImVec4 { x: 0.3, y: 0.3, z: 0.3, w: 1.0 })}
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


pub trait ImgGuiGlue {
    fn render(&mut self);
    fn get_value(&self) -> Option<bool>;
}

impl ImgGuiGlue for Checkbox {
    fn render(&mut self) {
        let mut label = self.label.clone(); 
        label.push('\0');
        unsafe {ImGui_Checkbox(label.as_ptr(), &mut self.value);}
    }

    fn get_value(&self) -> Option<bool> {
        Some(self.value)
    }
}

impl<T: ImgGuiGlue> ImgGuiGlue for Window<T> {
    fn render(&mut self) {
        for item in &mut self.items {
            item.render();
        }
    }    

    fn get_value(&self) -> Option<bool>{
        None
    }
}

pub struct Checkbox {
    label: String,
    value: bool,
}

impl Checkbox {
    pub fn new(label: String) -> Checkbox {
        Checkbox { label, value: false }
    }
}

impl<T: ImgGuiGlue> Window<T> {
    pub fn new() -> Window<T> {
        Window { items: vec![] }
    }

    pub fn append(&mut self, item: T) {
        self.items.push(item);
    }
}