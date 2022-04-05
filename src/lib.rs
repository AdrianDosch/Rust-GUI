mod imgui;
use std::ffi::c_void;

use imgui::*;

pub struct GUI<'a> {
    pub windows: Vec<&'a Window<'a>>,
    glfw_window: &'static c_void,
    io: &'static c_void,
    should_close: bool,
}

pub struct Window<'a> {
    pub items: Vec<&'a dyn ImgGuiGlue>,
    show: bool,
    name: String
}

impl<'a> Window<'a> {
    pub fn new(name: String) -> Window<'a> {
        Window { items: vec![], show: true, name}
    }

    pub fn append<T: ImgGuiGlue + 'a>(&mut self, item: &'a T)  {
        self.items.push(item);
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

impl<'a> ImgGuiGlue for Window<'a> {
    fn render(&self) {
        if self.show {
            for item in &self.items {
                let mut name = self.name.clone();
                if name.len() == 0 {
                    name = " ".into();
                }
                name.push('\0');
                unsafe {ImGui_Begin(name.as_ptr(), &self.show)}
                item.render();
                unsafe {ImGui_End()}
            }
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
        if self.value {
            let x = self.callback;
            x();
        }
    }

    fn get_value(&self) -> Option<bool> {
        Some(self.value)
    }
}

pub struct Color {
    pub col: ImVec4,
    label: String
}

impl Color {
    pub fn new() -> Color {
        let col = ImVec4{ x: 0.3, y: 0.3, z: 0.3, w: 0.1};
        Color { col, label: " ".into() }
    }
}

impl ImgGuiGlue for Color {
    fn render(&self) {
        let mut label = self.label.clone();
        label.push('\0');
        unsafe {ImGui_ColorEdit3(label.as_ptr(), &self.col)}
    }

    fn get_value(&self) -> Option<bool> {
        None
    }
}

pub fn show_demo_window() {
    unsafe{ imgui::show_demo_window()}
}


