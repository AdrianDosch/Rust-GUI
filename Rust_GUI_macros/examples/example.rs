use rust_gui_macros::*;

fn main() {
    what!(SliderInt, ImGui_SliderInt(self.label.blocking_write().as_ptr(), &self.value, self.min_val, self.max_val), *self.callback.blocking_read(), *self.value.blocking_read());
}