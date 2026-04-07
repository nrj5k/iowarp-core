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

//! Aneris Profiler - Combined Subprocess + Telemetry Capture
//!
//! This binary spawns a subprocess with CTE I/O interception and captures
//! telemetry in real-time, displaying results as they occur.
//!
//! Usage:
//!   LD_LIBRARY_PATH=~/clio-core/build/bin:$LD_LIBRARY_PATH \
//!     CHI_WITH_RUNTIME=1 \
//!     aneris-profiler <command> [args...]
//!
//! Example:
//!   aneris-profiler ior -t 1m -b 16m -s 16

use std::process::{Command, Stdio};
use std::time::Duration;
use wrp_cte::sync::init;

fn main() {
    // Parse command line
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <command> [args...]", args[0]);
        eprintln!("Example: {} ior -t 1m -b 16m -s 16", args[0]);
        std::process::exit(1);
    }

    let executable = &args[1];
    let exec_args = &args[2..];

    println!("=== Aneris Profiler ===");
    println!("Executable: {}", executable);
    println!("");

    // Initialize CTE
    println!("[1/3] Initializing CTE runtime...");
    if let Err(e) = init("") {
        eprintln!("Failed to initialize CTE: {}", e);
        std::process::exit(1);
    }
    println!("      ✓ CTE runtime initialized\n");

    // Get build directory from compile-time configuration
    #[cfg(aneris_build_dir)]
    const BUILD_DIR: &str = aneris_build_dir;

    #[cfg(not(aneris_build_dir))]
    const BUILD_DIR: &str = "/tmp/iowarp-build"; // fallback

    // Get paths - use compile-time configured build directory or environment override
    let build_dir = std::env::var("IOWARP_BUILD_DIR").unwrap_or_else(|_| BUILD_DIR.to_string());
    let posix_adapter = format!("{}/lib/libwrp_cte_posix.so", build_dir);

    // Check adapter
    if !std::path::Path::new(&posix_adapter).exists() {
        eprintln!("[!] Warning: POSIX adapter not found at {}", posix_adapter);
        eprintln!("    I/O interception will not work.");
    } else {
        println!("[✓] POSIX adapter found");
    }

    // Give runtime time to initialize
    std::thread::sleep(Duration::from_millis(100));

    // Spawn subprocess with LD_PRELOAD
    println!("[2/3] Starting subprocess with I/O interception...");
    let mut child = Command::new(executable)
        .args(exec_args)
        .env("LD_PRELOAD", &posix_adapter)
        .env(
            "LD_LIBRARY_PATH",
            format!(
                "{}/bin:{}",
                build_dir,
                std::env::var("LD_LIBRARY_PATH").unwrap_or_default()
            ),
        )
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("Failed to spawn subprocess");

    println!("      ✓ Subprocess started (PID: {})\n", child.id());
    println!("=== Telemetry Capture Active ===\n");

    // Wait for subprocess
    let status = child.wait().expect("Failed to wait for subprocess");

    // Give runtime time to catch final operations
    std::thread::sleep(Duration::from_millis(500));

    // Poll and display final results
    println!("\n=== Telemetry Summary ===");
    // Note: In a real implementation, you would poll telemetry during execution
    // For this simplified example, we just report subprocess exit status
    println!("Subprocess completed. Telemetry polling not implemented in this example.");

    println!("\nSubprocess exited with: {:?}", status.code());
}
