use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=Accento.manifest");
    println!("cargo:rerun-if-changed=build.rs");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    // The mark is drawn from original geometric shapes, with no external artwork or fonts.
    let mut image = vec![0u8; 40];
    image[0..4].copy_from_slice(&40u32.to_le_bytes());
    image[4..8].copy_from_slice(&32u32.to_le_bytes());
    image[8..12].copy_from_slice(&64u32.to_le_bytes());
    image[12..14].copy_from_slice(&1u16.to_le_bytes());
    image[14..16].copy_from_slice(&32u16.to_le_bytes());
    for y in (0..32).rev() {
        for x in 0..32 {
            let fx = x as f32;
            let fy = y as f32;
            let rounded = ((fx - 15.5).abs() - 10.)
                .max(0.)
                .hypot(((fy - 15.5).abs() - 10.).max(0.))
                < 5.5;
            let letter = (9..=23).contains(&y)
                && ((fx - (16. - (fy - 9.) * 0.43)).abs() < 1.4
                    || (fx - (16. + (fy - 9.) * 0.43)).abs() < 1.4
                    || (17..=19).contains(&y) && (12..=20).contains(&x));
            let accent = (5..=7).contains(&y) && (18..=22).contains(&x);
            image.extend_from_slice(if letter || accent {
                &[255, 255, 255, 255]
            } else if rounded {
                &[224, 100, 42, 255]
            } else {
                &[0, 0, 0, 0]
            });
        }
    }
    image.extend_from_slice(&[0u8; 128]);
    let mut ico = vec![0, 0, 1, 0, 1, 0, 32, 32, 0, 0, 1, 0, 32, 0];
    ico.extend_from_slice(&(image.len() as u32).to_le_bytes());
    ico.extend_from_slice(&22u32.to_le_bytes());
    ico.extend(image);
    let icon = out.join("Accento.ico");
    fs::write(&icon, ico).unwrap();
    tauri_build::try_build(
        tauri_build::Attributes::new().windows_attributes(
            tauri_build::WindowsAttributes::new()
                .window_icon_path(&icon)
                .app_manifest(include_str!("Accento.manifest")),
        ),
    )
    .expect("Impossibile preparare le risorse Tauri");
}
