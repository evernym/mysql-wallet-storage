use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();
    println!("target={}", target);

    if target.find("-windows-").is_some() {
        let profile = env::var("PROFILE").unwrap();
        println!("profile={}", profile);

        let indy_dir = env::var("AURORA_STORAGE_DIR").unwrap_or(format!("..\\libaurorawallet\\target\\{}", profile));
        println!("aurora_storage_dir={}", indy_dir);

        println!("cargo:rustc-link-lib=dylib=aurorastorage.dll");
        println!("cargo:rustc-flags=-L {}", indy_dir);
    }
}
