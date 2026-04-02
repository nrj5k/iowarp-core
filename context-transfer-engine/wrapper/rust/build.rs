fn main() {
    // Get include and library paths from environment (set by CMake/Corrosion)
    // Fall back to defaults for standalone cargo builds
    let include_dir =
        std::env::var("IOWARP_INCLUDE_DIR").unwrap_or_else(|_| "/usr/local/include".to_string());
    let lib_dir = std::env::var("IOWARP_LIB_DIR").unwrap_or_else(|_| "/usr/local/lib".to_string());

    // Build the CXX bridge and C++ shim
    cxx_build::bridge("src/ffi.rs")
        .file("shim/shim.cc")
        .std("c++20")
        // Coroutine support
        .flag("-fcoroutines")
        // Include paths
        .include(&include_dir)
        .include(".") // for shim/shim.h
        // HSHM defines (match CMake build)
        .define("HSHM_ENABLE_CEREAL", "1")
        .define("HSHM_ENABLE_ZMQ", "1")
        .define("HSHM_LOG_LEVEL", "0")
        // Suppress warnings from CTE/chimaera headers
        .flag("-Wno-unused-parameter")
        .flag("-Wno-unused-variable")
        .flag("-Wno-missing-field-initializers")
        .flag("-Wno-sign-compare")
        .flag("-Wno-reorder")
        .flag("-Wno-pedantic")
        .compile("cte_shim");

    // Library search paths
    println!("cargo:rustc-link-search=native={}", lib_dir);

    // Link to CTE and dependencies
    println!("cargo:rustc-link-lib=dylib=wrp_cte_core_client");
    println!("cargo:rustc-link-lib=dylib=chimaera_cxx");
    println!("cargo:rustc-link-lib=dylib=hermes_shm_host");
    println!("cargo:rustc-link-lib=dylib=zmq");

    // RPATH for relocatable builds
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_dir);

    // Rebuild triggers
    println!("cargo:rerun-if-changed=shim/shim.h");
    println!("cargo:rerun-if-changed=shim/shim.cc");
    println!("cargo:rerun-if-changed=src/ffi.rs");
    println!("cargo:rerun-if-env-changed=IOWARP_INCLUDE_DIR");
    println!("cargo:rerun-if-env-changed=IOWARP_LIB_DIR");
}
