fn main() {
    std::process::Command::new("/bin/qemu-system-x86_64")
        .arg("./target/debug/myos.img")
        .spawn()
        .unwrap();
}
