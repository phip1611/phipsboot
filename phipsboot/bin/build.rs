fn main() {
    let linker_script = "link.ld";
    // Weird: This path must not include the "bin/":
    let rerun_if_changed_path = linker_script;
    //        but this path must include the "bin/":
    let linker_arg_path = format!("bin/{linker_script}");

    println!("cargo:rerun-if-changed={rerun_if_changed_path}");
    println!("cargo:rustc-link-arg=-T{linker_arg_path}");
}
