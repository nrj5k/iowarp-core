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

//! CXX bridge to C++ CTE library
//!
//! This module defines the FFI boundary between Rust and C++ using the cxx crate.
//! All shared structs MUST match the C++ layout exactly.

/// The CXX bridge module
///
/// This bridge defines:
/// - Shared structs with identical memory layout to C++
/// - Opaque C++ types that Rust can hold but not inspect
/// - Functions exported from C++ to Rust
#[cxx::bridge(namespace = "cte_ffi")]
pub mod ffi {
    //==========================================================================
    // Shared Structs (must match C++ layout exactly)
    //==========================================================================

    /// CTE Tag ID - unique identifier for tags/blobs
    ///
    /// Layout MUST match chi::UniqueId (8 bytes):
    /// - major: u32 (major identifier)
    /// - minor: u32 (minor identifier)
    #[derive(Clone, Debug)]
    pub struct CteTagId {
        pub major: u32,
        pub minor: u32,
    }

    /// Steady clock time point
    ///
    /// Represents std::chrono::steady_clock::time_point
    /// Stores nanoseconds since arbitrary epoch
    #[derive(Clone, Debug)]
    pub struct SteadyTime {
        pub nanos: i64,
    }

    /// Telemetry entry for CTE operations
    ///
    /// Contains metadata about operations for monitoring
    #[derive(Clone, Debug)]
    pub struct CteTelemetry {
        /// Operation type as u32 (CteOp enum value)
        pub op: u32,
        /// Offset in blob
        pub off: u64,
        /// Size of operation
        pub size: u64,
        /// Tag ID associated with operation
        pub tag_id: CteTagId,
        /// Modification time (steady clock)
        pub mod_time: SteadyTime,
        /// Read time (steady clock)
        pub read_time: SteadyTime,
        /// Logical time counter
        pub logical_time: u64,
    }

    //==========================================================================
    // Opaque C++ Types
    //==========================================================================

    unsafe extern "C++" {
        include!("shim/shim.h");

        type Client;

        /// Tag handle for blob operations
        ///
        /// Wraps cte_ffi::CteTag which wraps wrp_cte::core::Tag
        pub type Tag;

        /// Pool query for routing
        ///
        /// Wraps chi::PoolQuery
        pub type PoolQuery;

        /// Include the C++ header
        include!("shim/shim.h");

        //======================================================================
        // Initialization
        //======================================================================

        /// Initialize CTE with embedded runtime
        ///
        /// # Arguments
        /// * `config_path` - Path to config file, or "" for defaults
        ///
        /// # Returns
        /// 0 on success, non-zero error code on failure
        pub fn cte_init(config_path: &str) -> i32;

        //======================================================================
        // Client Operations
        //======================================================================

        /// Create a new CTE client
        ///
        /// # Returns
        /// UniquePtr to new Client
        pub fn client_new() -> UniquePtr<Client>;

        /// Poll telemetry log from CTE
        ///
        /// # Arguments
        /// * `client` - The CTE client
        /// * `min_time` - Minimum timestamp to fetch (0 for all)
        ///
        /// # Returns
        /// Vector of telemetry entries
        pub fn client_poll_telemetry(client: &Client, min_time: u64) -> Vec<CteTelemetry>;

        /// Reorganize a blob (change placement score)
        ///
        /// # Arguments
        /// * `client` - The CTE client
        /// * `major` - Tag ID major component
        /// * `minor` - Tag ID minor component
        /// * `name` - Blob name
        /// * `score` - New placement score (0.0-1.0)
        ///
        /// # Returns
        /// 0 on success, non-zero error code
        pub fn client_reorganize_blob(
            client: &Client,
            major: u32,
            minor: u32,
            name: &str,
            score: f32,
        ) -> i32;

        /// Delete a blob
        ///
        /// # Arguments
        /// * `client` - The CTE client
        /// * `major` - Tag ID major component
        /// * `minor` - Tag ID minor component
        /// * `name` - Blob name
        ///
        /// # Returns
        /// 0 on success, non-zero error code
        pub fn client_del_blob(client: &Client, major: u32, minor: u32, name: &str) -> i32;

        //======================================================================
        // Pool Query Constructors (Factory Functions)
        //======================================================================

        /// Create a broadcast pool query
        ///
        /// # Arguments
        /// * `timeout` - Network timeout in seconds
        pub fn pool_query_broadcast(timeout: f32) -> UniquePtr<PoolQuery>;

        /// Create a dynamic pool query
        ///
        /// # Arguments
        /// * `timeout` - Network timeout in seconds
        pub fn pool_query_dynamic(timeout: f32) -> UniquePtr<PoolQuery>;

        /// Create a local pool query
        pub fn pool_query_local() -> UniquePtr<PoolQuery>;

        //======================================================================
        // Tag Operations
        //======================================================================

        /// Create or get a tag by name
        ///
        /// # Arguments
        /// * `name` - Tag name
        pub fn tag_new(name: &str) -> UniquePtr<Tag>;

        /// Open an existing tag by ID
        ///
        /// # Arguments
        /// * `major` - Tag ID major component
        /// * `minor` - Tag ID minor component
        pub fn tag_from_id(major: u32, minor: u32) -> UniquePtr<Tag>;

        /// Get the placement score of a blob
        ///
        /// # Arguments
        /// * `tag` - The tag
        /// * `name` - Blob name
        ///
        /// # Returns
        /// Score value (0.0-1.0)
        pub fn tag_get_blob_score(tag: &Tag, name: &str) -> f32;

        /// Reorganize a blob within a tag
        ///
        /// # Arguments
        /// * `tag` - The tag
        /// * `name` - Blob name
        /// * `score` - New placement score
        ///
        /// # Returns
        /// 0 on success, non-zero error code
        pub fn tag_reorganize_blob(tag: &Tag, name: &str, score: f32) -> i32;

        /// Write data into a blob
        ///
        /// # Arguments
        /// * `tag` - The tag
        /// * `name` - Blob name
        /// * `data` - Data to write
        /// * `offset` - Offset in blob
        /// * `score` - Placement score
        pub fn tag_put_blob(tag: &Tag, name: &str, data: &[u8], offset: u64, score: f32);

        /// Read data from a blob
        ///
        /// # Arguments
        /// * `tag` - The tag
        /// * `name` - Blob name
        /// * `size` - Number of bytes to read
        /// * `offset` - Offset in blob
        ///
        /// # Returns
        /// Vector of bytes read
        pub fn tag_get_blob(tag: &Tag, name: &str, size: u64, offset: u64) -> Vec<u8>;

        /// Get the size of a blob
        ///
        /// # Arguments
        /// * `tag` - The tag
        /// * `name` - Blob name
        ///
        /// # Returns
        /// Size in bytes
        pub fn tag_get_blob_size(tag: &Tag, name: &str) -> u64;

        /// List all blobs in a tag
        ///
        /// # Arguments
        /// * `tag` - The tag
        ///
        /// # Returns
        /// Vector of blob names
        pub fn tag_get_contained_blobs(tag: &Tag) -> Vec<String>;
    }
}

//==============================================================================
// Type Conversions
//==============================================================================

use crate::types::{CteOp, CteTagId as RustCteTagId, SteadyTime as RustSteadyTime};

impl From<&ffi::CteTagId> for RustCteTagId {
    fn from(id: &ffi::CteTagId) -> Self {
        RustCteTagId::new(id.major, id.minor)
    }
}

impl From<&RustCteTagId> for ffi::CteTagId {
    fn from(id: &RustCteTagId) -> Self {
        ffi::CteTagId {
            major: id.major,
            minor: id.minor,
        }
    }
}

impl From<&ffi::SteadyTime> for RustSteadyTime {
    fn from(time: &ffi::SteadyTime) -> Self {
        RustSteadyTime::from_nanos(time.nanos)
    }
}

impl From<u32> for CteOp {
    fn from(value: u32) -> Self {
        match value {
            0 => CteOp::PutBlob,
            1 => CteOp::GetBlob,
            2 => CteOp::DelBlob,
            3 => CteOp::GetOrCreateTag,
            4 => CteOp::DelTag,
            5 => CteOp::GetTagSize,
            _ => CteOp::PutBlob, // Default fallback
        }
    }
}

impl From<CteOp> for u32 {
    fn from(op: CteOp) -> Self {
        op as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CteOp, CteTagId, SteadyTime};

    #[test]
    fn test_cte_tag_id_conversion() {
        let rust_id = CteTagId::new(1, 2);
        let ffi_id: ffi::CteTagId = (&rust_id).into();

        assert_eq!(ffi_id.major, 1);
        assert_eq!(ffi_id.minor, 2);

        let back: CteTagId = (&ffi_id).into();
        assert_eq!(back.major, 1);
        assert_eq!(back.minor, 2);
    }

    #[test]
    fn test_steady_time_conversion() {
        let rust_time = SteadyTime::from_nanos(1234567890);
        let ffi_time: ffi::SteadyTime = (&rust_time).into();

        assert_eq!(ffi_time.nanos, 1234567890);

        let back: SteadyTime = (&ffi_time).into();
        assert_eq!(back.nanos, 1234567890);
    }

    #[test]
    fn test_cte_op_conversion() {
        assert_eq!(CteOp::from(0), CteOp::PutBlob);
        assert_eq!(CteOp::from(1), CteOp::GetBlob);
        assert_eq!(CteOp::from(2), CteOp::DelBlob);
        assert_eq!(CteOp::from(3), CteOp::GetOrCreateTag);
        assert_eq!(CteOp::from(4), CteOp::DelTag);
        assert_eq!(CteOp::from(5), CteOp::GetTagSize);
        assert_eq!(CteOp::from(999), CteOp::PutBlob); // fallback

        let put: u32 = CteOp::PutBlob.into();
        assert_eq!(put, 0);
    }
}
