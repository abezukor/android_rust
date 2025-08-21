use std::path::PathBuf;
use std::{env, path::Path};

use android_build::{Dexer, JavaBuild};
use walkdir::WalkDir;

const COMMON_LIBRARY_PATH: &str =
    "../../java/java_rust_obj/src/main/java/com/maticrobots/java_rust_obj";
const JAVA_LIBRARY_DIR: &str = "../../java/nsd_rs/src/main/java/com/maticrobots/nsd_rs";

fn main() {
    if !env::var("TARGET").unwrap().contains("android") {
        return;
    }

    let common_library_path = Path::new(COMMON_LIBRARY_PATH);
    let java_deps = [common_library_path.join("RustArcBoxDynAny.java")];

    let java_library_dir = Path::new(JAVA_LIBRARY_DIR);

    let java_srcs = [
        java_library_dir.join("NSDDiscoveryListener.java"),
        java_library_dir.join("NSDServiceResolver.java"),
    ];

    let out_dir: PathBuf = env::var_os("OUT_DIR").unwrap().into();
    let out_class_dir = out_dir.join("java");

    if out_class_dir.try_exists().unwrap_or(false) {
        let _ = std::fs::remove_dir_all(&out_class_dir);
    }
    std::fs::create_dir_all(&out_class_dir)
        .unwrap_or_else(|e| panic!("Cannot create output directory {out_class_dir:?} - {e}"));

    let android_jar = android_build::android_jar(None).expect("No Android platforms found");

    // Compile the Java file into .class files
    let o = JavaBuild::new()
        .files(java_deps.iter().chain(java_srcs.iter()))
        .class_path(&android_jar)
        .classes_out_dir(&out_class_dir)
        .java_source_version(8)
        .java_target_version(8)
        .command()
        .unwrap_or_else(|e| panic!("Could not generate the java compiler command: {e}"))
        .args(["-encoding", "UTF-8"])
        .output()
        .unwrap_or_else(|e| panic!("Could not run the java compiler: {e}"));

    if !o.status.success() {
        panic!(
            "Java compilation failed: {}",
            String::from_utf8_lossy(&o.stderr)
        );
    }

    // Only output the srcs files, not their dependencies
    let output_class_names: Vec<&str> = java_srcs
        .iter()
        .map(|java_src| {
            java_src
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .strip_suffix(".java")
                .unwrap()
        })
        .collect();
    let output_class_files = WalkDir::new(&out_class_dir)
        .into_iter()
        .filter_map(|entry| {
            let entry = entry.unwrap();
            if !entry.file_type().is_file() {
                return None;
            }
            output_class_names
                .contains(
                    &entry
                        .file_name()
                        .to_str()
                        .unwrap()
                        .strip_suffix(".class")
                        .unwrap(),
                )
                .then_some(entry.path().to_owned())
        });
    let o = Dexer::new()
        .android_jar(&android_jar)
        .class_path(&out_class_dir)
        .files(output_class_files)
        .android_min_api(33)
        .out_dir(out_dir)
        .command()
        .unwrap_or_else(|e| panic!("Could not generate the D8 command: {e}"))
        .output()
        .unwrap_or_else(|e| panic!("Error running D8: {e}"));

    if !o.status.success() {
        panic!(
            "Dex conversion failed: {}",
            String::from_utf8_lossy(&o.stderr)
        );
    }

    for java_src in java_srcs {
        println!(
            "cargo:rerun-if-changed={}",
            java_src.as_os_str().to_str().unwrap()
        );
    }
}
