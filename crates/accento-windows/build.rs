use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=Accento.manifest");
    println!("cargo:rerun-if-changed=build.rs");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
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
    let rc = out.join("Accento.rc");
    let manifest = root.join("Accento.manifest");
    let resource = format!(
        "1 24 \"{}\"\n1 ICON \"{}\"\n1 VERSIONINFO\nFILEVERSION 0,2,0,0\nPRODUCTVERSION 0,2,0,0\nFILEOS 0x40004\nFILETYPE 1\nBEGIN\n BLOCK \"StringFileInfo\"\n BEGIN\n BLOCK \"040904b0\"\n BEGIN\n VALUE \"FileDescription\", \"Accento\"\n VALUE \"ProductName\", \"Accento\"\n VALUE \"FileVersion\", \"0.2.0\"\n END\n END\n BLOCK \"VarFileInfo\"\n BEGIN\n VALUE \"Translation\", 0x409, 1200\n END\nEND\n",
        manifest.display().to_string().replace('\\', "/"),
        icon.display().to_string().replace('\\', "/")
    );
    fs::write(&rc, resource).unwrap();
    let mut candidates = Vec::new();
    if let Some(bin) = env::var_os("WindowsSdkVerBinPath") {
        candidates.push(PathBuf::from(bin).join("x64/rc.exe"));
    }
    let sdk = env::var_os("WindowsSdkDir")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(
                env::var_os("ProgramFiles(x86)").unwrap_or_else(|| "C:/Program Files (x86)".into()),
            )
            .join("Windows Kits/10")
        });
    if let Ok(entries) = fs::read_dir(sdk.join("bin")) {
        let mut versions: Vec<_> = entries
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| {
                p.file_name()
                    .is_some_and(|n| n.to_string_lossy().starts_with("10."))
            })
            .collect();
        versions.sort();
        versions.reverse();
        candidates.extend(versions.into_iter().map(|p| p.join("x64/rc.exe")));
    }
    let compiler = candidates
        .into_iter()
        .find(|p| p.is_file())
        .unwrap_or_else(|| PathBuf::from("rc.exe"));
    let res = out.join("Accento.res");
    let status = Command::new(compiler)
        .arg("/nologo")
        .arg("/fo")
        .arg(&res)
        .arg(&rc)
        .status()
        .expect("Installare Windows SDK con Resource Compiler (rc.exe)");
    assert!(status.success(), "Impossibile compilare le risorse Windows");
    println!("cargo:rustc-link-arg-bin=Accento={}", res.display());
}
