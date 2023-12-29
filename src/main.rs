fn main() {
    std::process::Command::new("/bin/qemu-system-x86_64")
        .arg("./target/debug/rosa-os.img")
        .arg("-serial")
        .arg("stdio")
        .spawn()
        .unwrap();
}
