mod imgui;
use std::ffi::c_void;

use imgui::*;

pub struct GUI<'a> {
    pub windows: Vec<&'a Window>,
    glfw_window: &'static c_void,
    io: &'static c_void,
    should_close: bool,
}

pub struct Window {
    pub items: Vec<Box<dyn ImgGuiGlue>>,
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
            let window_handle = init_gui();
            GUI {
                windows: vec![],
                glfw_window: window_handle.window,
                io: window_handle.io,
                should_close: false,
            }
        }
    }

    pub fn add_window(&mut self, window: &'a Window) {
        self.windows.push(window);
    }

    pub fn update(&mut self) {
        unsafe {
            start_frame();
        }
        for i in 0..self.windows.len() {
            self.windows[i].render();
        }
        unsafe {
            end_frame(
                self.glfw_window,
                self.io,
                ImVec4 {
                    x: 0.3,
                    y: 0.3,
                    z: 0.3,
                    w: 1.0,
                },
            );
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
        unsafe {
            ImGui_Checkbox(label.as_ptr(), &self.value);
        }
        if self.value {
            let x = self.callback;
            x();
        }
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

    fn get_value(&self) -> Option<bool> {
        None
    }
}

pub struct Checkbox {
    label: String,
    value: bool,
    callback: fn() -> (),
}

impl Checkbox {
    pub fn new(label: String) -> Checkbox {
        Checkbox {
            label,
            value: false,
            callback: ||{},
        }
    }

    pub fn set_callback(&mut self, callback: fn() -> ()) {
        self.callback = callback;
    }
}

pub struct Text {
    pub text: String,
}

impl Text {
    pub fn new(text: String) -> Text {
        Text { text }
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

pub struct Button {
    pub text: String,
    pub value: bool,
    callback: fn() -> (),
}

impl Button {
    pub fn new(text: String) -> Button {
        Button {
            text,
            value: false,
            callback: || {},
        }
    }

    pub fn set_callback(&mut self, callback: fn() -> ()) {
        self.callback = callback;
    }
}

impl ImgGuiGlue for Button {
    fn render(&self) {
        let mut text = self.text.clone();
        text.push('\0');
        unsafe {
            ImGui_Button(text.as_ptr(), &self.value);
        }
        if self.value == true {
            let x = self.callback;
            x();
        }
    }

    fn get_value(&self) -> Option<bool> {
        Some(self.value)
    }
}

pub fn show_demo_window() {
    unsafe{ imgui::show_demo_window()}
}
