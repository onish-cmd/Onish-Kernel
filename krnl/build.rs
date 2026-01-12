// krnl/build.rs
fn main() {
    // Tell Cargo to re-run this if boot.S changes
    println!("cargo:rerun-if-changed=src/boot.S");
    
    // Use the 'cc' crate to assemble boot.S
    cc::Build::new()
        .file("src/boot.S")
        .compile("boot");
}