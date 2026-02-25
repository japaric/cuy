fn main() {
    println!("cargo:rustc-link-arg=-Tlayout.ld"); // from rt crate
    println!("cargo:rustc-link-arg=-Tmemory.ld"); // from qemu-virt crate
}
