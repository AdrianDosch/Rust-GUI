mod imgui;
use std::{ffi::{self, c_void}};

use imgui::*;

pub struct GUI<'a> {
    pub windows: Vec<&'a Window>,
    glfw_window: &'static c_void,
    should_close: bool,
}

pub struct Window {
    pub items: Vec<Box<dyn ImgGuiGlue>>
}

impl Window {
    pub fn new() -> Window {
        Window { items: vec![] }
    }

    pub fn append<T: ImgGuiGlue + 'static>(&mut self, item: T) {
        self.items.push(Box::new(item));
    }
}

impl<'a> GUI<'a> {
    pub fn new() -> GUI<'a> {
        unsafe {
            let window_handle = init_gui1();
            GUI { windows: vec![], glfw_window: window_handle.window, should_close: false}
        }
    }

    pub fn add_window(&mut self, window: &'a Window) {
        self.windows.push(window);
    }

    pub fn update(&mut self) {
        unsafe {start_frame1();}
        for i in 0..self.windows.len() {
            self.windows[i].render();
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
    fn render(&self);
    fn get_value(&self) -> Option<bool> {
        None
    }
}

impl ImgGuiGlue for Checkbox {
    fn render(&self) {
        let mut label = self.label.clone(); 
        label.push('\0');
        unsafe {ImGui_Checkbox(label.as_ptr(), &self.value);}
    }

    fn get_value(&self) -> Option<bool> {
        Some(self.value)
    }
}

impl ImgGuiGlue for Window {
    fn render(&self) {
        for item in &self.items {
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

pub struct Text {
    pub text: String,
}

impl Text {
    pub fn new(text: String) -> Text{
        Text { text }
    }
}

impl ImgGuiGlue for Text {
    fn render(&self) {
        let mut text = self.text.clone(); 
        text.push('\0');
        unsafe {ImGui_Text(text.as_ptr());}
    }
}

pub struct Button {
    pub text: String,
    pub value: bool,
}

impl Button {
    pub fn new(text: String) -> Button {
        Button { text, value: false }
    }
}

impl ImgGuiGlue for Button {
    fn render(&self) {
        let mut text = self.text.clone(); 
        text.push('\0');
        unsafe { ImGui_Button(text.as_ptr(), &self.value);}
    }

    fn get_value(&self) -> Option<bool> {
        Some(self.value)
    }
}