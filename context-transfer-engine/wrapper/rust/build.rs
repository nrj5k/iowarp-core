/*
 * Copyright (c) 2024, Gnosis Research Center, Illinois Institute of Technology
 * All rights reserved.
 *
 * This file is part of IOWarp Core.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are met:
 *
 * 1. Redistributions of source code must retain the above copyright notice,
 *    this list of conditions and the following disclaimer.
 *
 * 2. Redistributions in binary form must reproduce the above copyright notice,
 *    this list of conditions and the following disclaimer in the documentation
 *    and/or other materials provided with the distribution.
 *
 * 3. Neither the name of the copyright holder nor the names of its
 *    contributors may be used to endorse or promote products derived from
 *    this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
 * ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE
 * LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
 * CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
 * SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
 * INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
 * CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
 * ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
 * POSSIBILITY OF SUCH DAMAGE.
 */

use std::env;
use std::path::Path;

/// Parses a semicolon-separated list of library specifications.
/// Each item can be:
/// - A library name (e.g., "zmq")
/// - A full path to a library (e.g., "/usr/lib/libzmq.so.5")
/// - A colon-separated list for static linking (e.g., "zmq;stdc++;gcc_s")
fn parse_zmq_libs(libs_var: &str) -> Vec<String> {
    libs_var
        .split(';')
        .filter(|s| !s.is_empty())
        .map(|s| s.trim().to_string())
        .collect()
}

/// Parses a colon-separated list of library directories.
fn parse_zmq_lib_dirs(dirs_var: &str) -> Vec<String> {
    dirs_var
        .split(':')
        .filter(|s| !s.is_empty())
        .map(|s| s.trim().to_string())
        .collect()
}

/// Determines if a string is a path (contains / or \ or .)
fn is_library_path(spec: &str) -> bool {
    spec.contains('/') || spec.contains('\\') || spec.contains('.')
}

/// Links a library by name (extracts library name from path if needed).
fn link_library(lib_spec: &str) {
    if is_library_path(lib_spec) {
        // It's a path - we need to extract the library name
        let path = Path::new(lib_spec);
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(lib_spec);

        // Remove common library prefixes and suffixes
        // libzmq.so.5 -> zmq
        // libzmq.so -> zmq
        // zmq.so -> zmq
        let lib_name = filename
            .strip_prefix("lib")
            .unwrap_or(filename)
            .split('.')
            .next()
            .unwrap_or(filename);

        // Add the directory containing the library to search path
        if let Some(parent) = path.parent() {
            if let Some(parent_str) = parent.to_str() {
                if !parent_str.is_empty() {
                    println!("cargo:rustc-link-search=native={}", parent_str);
                }
            }
        }

        println!("cargo:rustc-link-lib=dylib={}", lib_name);
    } else {
        // It's just a library name
        println!("cargo:rustc-link-lib=dylib={}", lib_spec);
    }
}

/// Attempts to link ZMQ from environment variables set by CMake.
/// Returns true if successful, false if fallback should be used.
fn try_link_zmq_from_cmake() -> bool {
    let zmq_libs = env::var("IOWARP_ZMQ_LIBS").unwrap_or_default();
    let zmq_lib_dirs = env::var("IOWARP_ZMQ_LIB_DIRS").unwrap_or_default();

    if zmq_libs.is_empty() {
        return false;
    }

    // Add library directories to search path
    for dir in parse_zmq_lib_dirs(&zmq_lib_dirs) {
        println!("cargo:rustc-link-search=native={}", dir);
    }

    // Link each library specification
    let libs = parse_zmq_libs(&zmq_libs);
    if libs.is_empty() {
        return false;
    }

    for lib in libs {
        link_library(&lib);
    }

    true
}

/// Attempts to link ZMQ using common library names for standalone cargo builds.
/// Tries common naming conventions as fallback.
fn link_zmq_fallback() {
    // Common ZMQ library names to try
    let fallback_names = vec!["zmq", "zmq:5"];

    let mut linked = false;
    for name in fallback_names {
        // Try to link and verify
        println!("cargo:rustc-link-lib=dylib={}", name);
        // Note: We can't verify at build time if the library exists,
        // so we just add it and let the linker fail with clear message if not found
        linked = true;
        break; // Use first successful candidate
    }

    if !linked {
        panic!(
            "ZeroMQ (libzmq) not found. Please either:\n\
             1. Build with CMake which will set IOWARP_ZMQ_LIBS and IOWARP_ZMQ_LIB_DIRS\n\
             2. Install libzmq development package:\n\
                - Ubuntu/Debian: sudo apt-get install libzmq3-dev\n\
                - CentOS/RHEL: sudo yum install zeromq-devel\n\
                - macOS: brew install zeromq\n\
             3. Set environment variables manually:\n\
                export IOWARP_ZMQ_LIBS=zmq\n\
                export IOWARP_ZMQ_LIB_DIRS=/usr/local/lib\n\
             4. Set IOWARP_LIB_DIR to directory containing libzmq.so\n"
        );
    }
}

fn main() {
    // Get include and library paths from environment (set by CMake/Corrosion)
    // Fall back to defaults for standalone cargo builds
    let include_dir =
        std::env::var("IOWARP_INCLUDE_DIR").unwrap_or_else(|_| "/usr/local/include".to_string());
    let lib_dir = std::env::var("IOWARP_LIB_DIR").unwrap_or_else(|_| "/usr/local/lib".to_string());

    // Additional include paths for chimaera and other dependencies
    // Multiple paths separated by colons
    let extra_includes = std::env::var("IOWARP_EXTRA_INCLUDES").unwrap_or_default();

    // Build the CXX bridge and C++ shim
    let mut build = cxx_build::bridge("src/ffi.rs");
    build
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
        .define("HSHM_ENABLE_PTHREADS", "1")
        .define("HSHM_ENABLE_OPENMP", "0")
        .define("HSHM_ENABLE_WINDOWS_THREADS", "0")
        .define("HSHM_DEFAULT_THREAD_MODEL", "hshm::thread::Pthread")
        .define("HSHM_DEFAULT_THREAD_MODEL_GPU", "hshm::thread::StdThread")
        .define("HSHM_LOG_LEVEL", "0")
        // Suppress warnings from CTE/chimaera headers
        .flag("-Wno-unused-parameter")
        .flag("-Wno-unused-variable")
        .flag("-Wno-missing-field-initializers")
        .flag("-Wno-sign-compare")
        .flag("-Wno-reorder")
        .flag("-Wno-pedantic");

    // Add extra include directories
    for path in extra_includes.split(':').filter(|s| !s.is_empty()) {
        build.include(path);
    }

    build.compile("cte_shim");

    // Library search paths
    println!("cargo:rustc-link-search=native={}", lib_dir);

    // Link to CTE and dependencies
    println!("cargo:rustc-link-lib=dylib=wrp_cte_core_client");
    println!("cargo:rustc-link-lib=dylib=chimaera_cxx");
    println!("cargo:rustc-link-lib=dylib=hermes_shm_host");

    // Dynamic ZMQ linking with CMake environment variables or fallback
    if !try_link_zmq_from_cmake() {
        link_zmq_fallback();
    }

    // RPATH for relocatable builds
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_dir);

    // Rebuild triggers
    println!("cargo:rerun-if-changed=shim/shim.h");
    println!("cargo:rerun-if-changed=shim/shim.cc");
    println!("cargo:rerun-if-changed=src/ffi.rs");
    println!("cargo:rerun-if-env-changed=IOWARP_INCLUDE_DIR");
    println!("cargo:rerun-if-env-changed=IOWARP_LIB_DIR");
    println!("cargo:rerun-if-env-changed=IOWARP_EXTRA_INCLUDES");
    println!("cargo:rerun-if-env-changed=IOWARP_ZMQ_LIBS");
    println!("cargo:rerun-if-env-changed=IOWARP_ZMQ_LIB_DIRS");
}
