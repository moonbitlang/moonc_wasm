fn main() {
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-lib=advapi32");
        println!("cargo:rustc-link-lib=tdh");
    }
}