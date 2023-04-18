use std::{process::Command, path::{PathBuf, Path}, env, fs, io};

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

fn verify_file_exists<P: AsRef<Path>>(path: P) -> io::Result<()> {
    if path.as_ref().try_exists()? {
        Ok(())
    } else {
        let parent = path.as_ref().parent().expect("No parent");
        println!("Listing directory: {}", parent.display());
        for dir in fs::read_dir(parent)? {
            println!("  {}", dir?.path().display());
        }
        let msg = format!("Path not found: {}", path.as_ref().display());
        Err(io::Error::new(io::ErrorKind::NotFound, msg))
    }
}

fn build_library() {
    // Should we always use Release of change to Debug when not building with --release?
    let build_type = "Release";
    let platform = "x64";

    let src_dir = root_dir().join("libwdi");
    let out_dir = src_dir.join(platform).join(build_type).join("lib");

    // Build libwdi using microsoft build tools:
    // MSBuild.exe libwdi.sln -p:Configuration=Release -p:Platform=x64
    let mut cmd = Command::new("MSBuild.exe");
    cmd.current_dir(src_dir);
    cmd.arg("libwdi.sln");
    cmd.arg("-m"); // use multiple concurrent processes
    cmd.arg(format!("-p:Configuration={build_type}"));
    cmd.arg(format!("-p:Platform={platform}"));

    // Generate build macros with #defines for custom library paths from env variables
    let lib_vars = ["WDK_DIR", "LIBUSB0_DIR", "LIBUSBK_DIR"];
    let mut defs = vec![];
    for var in lib_vars {
        println!("cargo:rerun-if-env-changed={var}");
        if let Ok(dir) = env::var(var) {
            let def = format!("{var}=\"{dir}\"");
            println!("Using: {def}");
            defs.push(def);
        }
    }
    if !defs.is_empty() {
        let macros = defs.join(";");
        cmd.arg(format!("-p:BuildMacros={macros}"));
    }

    // Run build
    println!("Running MSBuild: {cmd:?}");
    cmd.status().expect("Failed to build libwdi using MSBuild");

    // Fail if the output file doesn't exist for easier debugging
    verify_file_exists(out_dir.join("libwdi"))
        .expect("Library output file does not exist");

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
