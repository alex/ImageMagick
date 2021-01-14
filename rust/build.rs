use std::env;
use std::path::PathBuf;

fn main() {
    let mut clang_args = vec!["-I../".to_string()];
    if let Ok(cflags) = env::var("MAGICK_CLFAGS") {
        // XXX: THIS IS WRONG AND BROKEN IF YOU HAVE A PATH A SPACE OR ANYTHING
        // LIKE THAT. USE A REAL PARSER HERE.
        for flag in cflags.split(' ') {
            if flag.is_empty() {
                continue;
            }
            clang_args.push(flag.to_string());
        }
    }

    println!("cargo:rerun-if-changed=wrapper.h");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_args(clang_args.iter())
        .blacklist_item("FP_.*")
        .size_t_is_usize(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Error generating bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Error writing bindings");

    let mut b = cc::Build::new();
    b.file("wrapper.c");
    for arg in clang_args {
        b.flag(&arg);
    }
    b.compile("rust_imagemagick_wrapper")
}
