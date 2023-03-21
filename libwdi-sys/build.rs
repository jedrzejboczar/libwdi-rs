use std::{process::Command, path::PathBuf, env};

fn root_dir() -> PathBuf {
    let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    dir.into()
}

fn generate_ffi_bindings() {
    let header = root_dir().join("libwdi/libwdi/libwdi.h")
        .into_os_string().into_string().unwrap();

    let bindings = bindgen::Builder::default()
        .allowlist_function("^wdi_.*")
        .allowlist_type("^wdi_.*")
        .allowlist_var("^wdi_.*")
        .default_enum_style(bindgen::EnumVariation::ModuleConsts)
        .header(header)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate libwdi bindings");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Could not write bindings file");
}

fn build_library() {
    let src_dir = root_dir().join("libwdi");
    let out_dir = root_dir().join("libwdi/x64/Release/lib");

    // Build libwdi using microsoft build tools:
    // MSBuild.exe libwdi.sln -p:Configuration=Release -p:Platform=x64
    Command::new("MSBuild.exe")
        .current_dir(src_dir)
        .args(["libwdi.sln", "-p:Configuration=Release", "-p:Platform=x64"])
        .status()
        .expect("Failed to build libwdi using MSBuild");

    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=libwdi");
    // No "rerun-if-changed", just run msbuild every time as it does it's own cacheing.

    // Add libwdi link dependencies
    println!("cargo:rustc-link-lib=setupapi");
    println!("cargo:rustc-link-lib=ntdll");
    // println!("cargo:rustc-link-lib=newdev");
    println!("cargo:rustc-link-lib=user32");
    println!("cargo:rustc-link-lib=shell32");
    println!("cargo:rustc-link-lib=ole32");
}

fn main() {
    generate_ffi_bindings();
    build_library();
}
