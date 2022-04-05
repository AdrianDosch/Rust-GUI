use std::ffi::c_void;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Variables {
    pub color: ImVec4,
    pub window1: Window1,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Window1 {
    pub show_demo_window: bool,
    pub show_another_window: bool,
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct ImVec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

extern "C" {
    pub fn init_gui() -> GUI_handle<'static>;
    pub fn destroy_gui(window: &c_void) -> ();
    pub fn close_window(window: &c_void) -> bool;
    pub fn ImGui_Checkbox(label: *const u8, vale: &bool);
    pub fn ImGui_Text(text: *const u8);
    pub fn ImGui_Button(text: *const u8, value: &bool);
    pub fn start_frame();
    pub fn end_frame(window: &'static c_void, io: &'static c_void, color: ImVec4);
    pub fn show_demo_window();
}

#[repr(C)]
#[derive(Debug)]
pub struct GUI_handle<'a> {
    pub window: &'a c_void,
    pub io: &'a c_void,
}
