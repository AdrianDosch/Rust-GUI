use std::{path::Path, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=src/gui/gui_lib.cpp");
    let imgui_str = format!("{}{}", std::env::var("OUT_DIR").unwrap(), "/imgui/");
    let imgui_path = imgui_str.as_str();

    //clone Dear ImGui
    if !Path::new(imgui_path).exists() {
        Command::new("git")
            .args([
                "clone",
                "https://github.com/ocornut/imgui.git",
                "--branch",
                "docking",
                imgui_path,
            ])
            .status()
            .expect("cloning Dear ImGui failed");
    }
    Command::new("git")
        .args(["-C", imgui_path, "pull"])
        .status()
        .expect("pulling Dear ImGui failed");

    //compile Dear ImGui
    cc::Build::new()
        .cpp(true)
        .include(format!("{}{}", imgui_path, ""))
        .include(format!("{}backends", imgui_path))
        .include(format!("{}{}", imgui_path, "examples/libs/glfw/include"))
        .file("src/gui/gui_lib.cpp")
        .file(format!("{}imgui.cpp", imgui_path))
        .file(format!("{}imgui_draw.cpp", imgui_path))
        .file(format!("{}imgui_tables.cpp", imgui_path))
        .file(format!("{}imgui_widgets.cpp", imgui_path))
        .file(format!("{}backends/imgui_impl_opengl3.cpp", imgui_path))
        .file(format!("{}backends/imgui_impl_glfw.cpp", imgui_path))
        .file(format!("{}backends/imgui_impl_glfw.cpp", imgui_path))
        .file(format!("{}imgui_demo.cpp", imgui_path))
        .compile("gui_lib");

    //link everything
    println!("cargo:rustc-link-lib=glfw3");
    println!(
        "cargo:rustc-link-search={}examples/libs/glfw/lib-vc2010-64",
        imgui_path
    );

    println!("cargo:rustc-link-lib=gdi32");
    println!("cargo:rustc-link-lib=opengl32");
    println!("cargo:rustc-link-lib=shell32");
}
