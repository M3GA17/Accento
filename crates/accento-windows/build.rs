use std::env;

fn main() {
    println!("cargo:rerun-if-changed=Accento.manifest");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=icons/Accento.ico");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        tauri_build::build();
        return;
    }
    tauri_build::try_build(
        tauri_build::Attributes::new().windows_attributes(
            tauri_build::WindowsAttributes::new()
                .window_icon_path("icons/Accento.ico")
                .app_manifest(include_str!("Accento.manifest")),
        ),
    )
    .expect("Impossibile preparare le risorse Tauri");
}
