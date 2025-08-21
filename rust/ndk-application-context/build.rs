use std::{
    env,
    ffi::OsString,
    io::Write,
    path::{Path, PathBuf},
    process::Stdio,
};

const ANDROID_INITIALIZATION_C_LIB: &[u8] = include_bytes!("android_initialization_c_lib.ifs");
const SHARED_LIBRARY_NAME: &str = "rust_context_autoinitialization_setter";

fn main() {
    if !env::var("TARGET").unwrap().contains("android") {
        return;
    }
    let ifs = find_ifs().unwrap();

    let mut ifs = std::process::Command::new(ifs)
        .args(get_target_arch())
        .arg(format!(
            "--output-elf={}/lib{}.so",
            env::var("OUT_DIR").unwrap(),
            SHARED_LIBRARY_NAME
        ))
        .stdin(Stdio::piped())
        .spawn()
        .expect("Failed to launch llvm-ifs");
    let mut ifs_stdin = ifs.stdin.take().unwrap();
    ifs_stdin.write_all(ANDROID_INITIALIZATION_C_LIB).unwrap();
    drop(ifs_stdin);
    let ifs = ifs.wait().unwrap();
    if !ifs.success() {
        panic!("llvm ifs failed");
    }

    println!("cargo::rustc-link-search={}", env::var("OUT_DIR").unwrap());
    println!("cargo::rustc-link-lib=dylib={SHARED_LIBRARY_NAME}");
}

fn find_ifs() -> Option<OsString> {
    if let Ok(ndk_path) = env::var("ANDROID_NDK_HOME") {
        return find_ifs_from_ndk(Path::new(&ndk_path)).map(PathBuf::into_os_string);
    }

    if let Ok(sdk_path) = env::var("ANDROID_HOME") {
        let mut ndk_versions = std::fs::read_dir(PathBuf::from(sdk_path).join("ndk")).ok()?;
        return ndk_versions.find_map(|ndk_version| {
            let ndk_version = ndk_version.ok()?;
            find_ifs_from_ndk(&ndk_version.path()).map(PathBuf::into_os_string)
        });
    }

    Some(OsString::from(llvm_ifs_name()))
}

fn find_ifs_from_ndk(ndk_base: &Path) -> Option<PathBuf> {
    let toolchain_prebuilts = ndk_base.join("toolchains/llvm/prebuilt");
    let mut architectures = std::fs::read_dir(&toolchain_prebuilts).ok()?;

    let architecture = architectures
        .find(|architecture| {
            architecture
                .as_ref()
                .is_ok_and(|architecture| architecture.file_type().unwrap().is_dir())
        })?
        .ok()?;
    Some(architecture.path().join("bin").join(llvm_ifs_name()))
}

const fn llvm_ifs_name() -> &'static str {
    match cfg!(target_os = "windows") {
        true => "llvm-ifs.exe",
        false => "llvm-ifs",
    }
}

// It seems that llvm-ifs does not automaticially fill in the machine field when using
// --target. So we have to manually construct the target for android targets
fn get_target_arch() -> [String; 3] {
    match env::var("TARGET").unwrap().as_str() {
        "x86_64-linux-android" => {
            ["--arch=X86_64", "--endianness=little", "--bitwidth=64"].map(str::to_owned)
        }
        "i686-linux-android" => {
            ["--arch=X86_64", "--endianness=little", "--bitwidth=32"].map(str::to_owned)
        }
        "aarch64-linux-android" => {
            ["--arch=AARCH64", "--endianness=little", "--bitwidth=64"].map(str::to_owned)
        }
        "armv7-linux-androideabi" => {
            ["--arch=ARM", "--endianness=little", "--bitwidth=32"].map(str::to_owned)
        }
        unknown_target => [
            format!("--target={unknown_target}"),
            String::new(),
            String::new(),
        ],
    }
}
