use std::process::Command;

const ASM_FILE: &str = "asm/context.S";
const O_FILE: &str = "asm/context.o";
const LIB_FILE: &str = "asm/libcontext.a";

fn main() {
    Command::new("cc")
        .args(&[ASM_FILE, "-c", "-fPIC", "-o"])
        .arg(O_FILE)
        .status()
        .unwrap();
    Command::new("ar")
        .args(&["crus", LIB_FILE, O_FILE])
        .status()
        .unwrap();

    println!("cargo:rustc-link-search=native={}", "asm");
    println!("cargo:rustc-link-lib=static=context");
    println("cargo:rerun-if-changed=asm/context.S");
}
