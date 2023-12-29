use std::path::Path;

fn main() {
    let kernel_path =
        std::env::var_os("CARGO_BIN_FILE_KERNEL_kernel").expect("No kernel path found");

    let target_dir = get_cargo_target_dir().unwrap();
    bootloader::BiosBoot::new(Path::new(&kernel_path))
        .create_disk_image(&Path::new(&target_dir).join("myos.img"))
        .unwrap();
}

fn get_cargo_target_dir() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR")?);
    let profile = std::env::var("PROFILE")?;
    let mut target_dir = None;
    let mut sub_path = out_dir.as_path();
    while let Some(parent) = sub_path.parent() {
        if parent.ends_with(&profile) {
            target_dir = Some(parent);
            break;
        }
        sub_path = parent;
    }
    let target_dir = target_dir.ok_or("not found")?;
    Ok(target_dir.to_path_buf())
}
