use std::ffi::c_void;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct ImGui_Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[allow(unused)]
extern "C" {
    pub fn init_gui(window_label: *const u8) -> GUI_handle<'static>;
    pub fn destroy_gui(window: &c_void);
    pub fn close_window(window: &c_void) -> bool;

    pub fn start_frame();
    pub fn end_frame(window: &'static c_void, io: &'static c_void, color: ImGui_Vec4);
    pub fn show_demo_window();

    //to be able to use the derive macro ImGuiGlue the functions have to have the name ImGui_<struct name>.
    pub fn ImGui_Checkbox(label: *const u8, value: &bool);
    pub fn ImGui_Text(text: *const u8);
    pub fn ImGui_Button(text: *const u8, value: &bool);
    pub fn ImGui_Begin(name: *const u8, close: &bool, flags: i32);
    pub fn ImGui_End();
    pub fn ImGui_ColorEdit3(label: *const u8, value: &ImGui_Vec4); //alias ImGui_Color
    pub fn ImGui_SameLine(offset_from_start_x: f32, spacing: f32);
    pub fn ImGui_SliderInt(label: *const u8, value: &i32, min_val: i32, max_val: i32);
    pub fn ImGui_SliderFloat(label: *const u8, value: &f32, min_val: f32, max_val: f32);
    pub fn ImGui_InputText(label: *const u8, value: *const u8, buffer_size: i32, flags: i32);
}

#[allow(non_snake_case)]
pub unsafe fn ImGui_Color(label: *const u8, value: &ImGui_Vec4) {
    ImGui_ColorEdit3(label, value);
}

#[repr(C)]
#[derive(Debug)]
pub struct GUI_handle<'a> {
    pub window: &'a c_void,
    pub io: &'a c_void,
}

pub trait ImGuiGlue {
    fn render(&self);
}
