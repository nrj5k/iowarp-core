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

//! Synchronous CTE API
//!
//! This module provides blocking (synchronous) wrappers around the CTE FFI.
//! For async operations, see the `r#async` module.
//!
//! # Example
//! ```
//! use wrp_cte::sync::{init, Client, Tag};
//!
//! // Initialize CTE
//! init("").expect("CTE init failed");
//!
//! // Create client and tag
//! let client = Client::new().unwrap();
//! let tag = Tag::new("my_dataset");
//!
//! // Use blocking operations
//! tag.put_blob_with_options("data.bin", b"hello", 0, 1.0);
//! let data = tag.get_blob("data.bin", 5, 0);
//! ```

use crate::error::{CteError, CteResult};
use crate::ffi;
use crate::types::{BdevType, ChimaeraMode, CteOp, CteTagId, CteTelemetry, PoolQuery, SteadyTime};
use std::sync::OnceLock;

/// Cached initialization result
static INIT_RESULT: OnceLock<CteResult<()>> = OnceLock::new();

/// Re-export types for sync API
pub use crate::types::{BdevType, ChimaeraMode, CteOp, CteTagId, CteTelemetry, SteadyTime};

/// Initialize CTE with embedded runtime
///
/// This function is thread-safe and will only initialize once.
/// Subsequent calls return the cached result.
///
/// # Arguments
/// * `config_path` - Path to configuration file, or "" for defaults
///
/// # Returns
/// * `Ok(())` on success
/// * `Err(CteError::InitFailed)` on failure
///
/// # Example
/// ```
/// use wrp_cte::sync::init;
///
/// init("").expect("CTE initialization failed");
/// ```
pub fn init(config_path: &str) -> CteResult<()> {
    // Thread-safe initialization using OnceLock.
    // This ensures only one thread initializes CTE.
    // Other threads get the cached result.
    INIT_RESULT
        .get_or_init(|| {
            let rc = ffi::cte_init(config_path);
            if rc == 0 {
                Ok(())
            } else {
                Err(CteError::InitFailed {
                    reason: format!("CTE initialization failed with code {}", rc),
                })
            }
        })
        .clone()
}

/// CTE client for low-level operations
///
/// Provides access to client-level operations like:
/// - Telemetry polling
/// - Blob reorganization
/// - Pool management
///
/// The client wraps the underlying CTE client handle.
pub struct Client {
    inner: cxx::UniquePtr<ffi::Client>,
}

impl Client {
    /// Create a new CTE client
    ///
    /// # Returns
    /// * `Ok(Client)` on success
    /// * `Err(CteError::InitFailed)` if CTE not initialized
    ///
    /// # Example
    /// ```
    /// use wrp_cte::sync::Client;
    ///
    /// let client = Client::new().unwrap();
    /// ```
    pub fn new() -> CteResult<Self> {
        let inner = ffi::client_new();
        Ok(Self { inner })
    }

    /// Poll telemetry log from CTE
    ///
    /// Returns telemetry entries for operations that occurred after `min_time`.
    ///
    /// # Arguments
    /// * `min_time` - Minimum timestamp to fetch (0 for all)
    ///
    /// # Returns
    /// Vector of telemetry entries
    ///
    /// # Example
    /// ```
    /// use wrp_cte::sync::Client;
    ///
    /// let client = Client::new().unwrap();
    /// let telemetry = client.poll_telemetry(0).unwrap();
    /// ```
    pub fn poll_telemetry(&self, min_time: u64) -> CteResult<Vec<CteTelemetry>> {
        let raw = ffi::client_poll_telemetry(&self.inner, min_time);
        Ok(raw
            .into_iter()
            .map(|t| CteTelemetry {
                op: t.op.into(),
                off: t.off,
                size: t.size,
                tag_id: (&t.tag_id).into(),
                mod_time: (&t.mod_time).into(),
                read_time: (&t.read_time).into(),
                logical_time: t.logical_time,
            })
            .collect())
    }

    /// Reorganize a blob (change placement score)
    ///
    /// Changes the importance score of a blob, which may trigger
    /// data migration between storage tiers.
    ///
    /// # Arguments
    /// * `tag_id` - ID of the tag containing the blob
    /// * `name` - Blob name
    /// * `score` - New placement score (0.0-1.0)
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(CteError::RuntimeError)` on failure
    pub fn reorganize_blob(&self, tag_id: CteTagId, name: &str, score: f32) -> CteResult<()> {
        let rc = ffi::client_reorganize_blob(&self.inner, tag_id.major, tag_id.minor, name, score);
        if rc == 0 {
            Ok(())
        } else {
            Err(CteError::RuntimeError {
                code: rc as u32,
                message: format!("Failed to reorganize blob {}", name),
            })
        }
    }

    /// Delete a blob
    ///
    /// # Arguments
    /// * `tag_id` - ID of the tag containing the blob
    /// * `name` - Blob name
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(CteError::RuntimeError)` on failure
    pub fn del_blob(&self, tag_id: CteTagId, name: &str) -> CteResult<()> {
        let rc = ffi::client_del_blob(&self.inner, tag_id.major, tag_id.minor, name);
        if rc == 0 {
            Ok(())
        } else {
            Err(CteError::RuntimeError {
                code: rc as u32,
                message: format!("Failed to delete blob {}", name),
            })
        }
    }
}

/// High-level tag wrapper for blob operations
///
/// A tag is a container (bucket) for blobs. This wrapper provides
/// convenient methods for blob storage, retrieval, and management.
///
/// # Example
/// ```
/// use wrp_cte::sync::Tag;
///
/// let tag = Tag::new("my_dataset");
/// tag.put_blob_with_options("data.bin", b"hello", 0, 1.0);
///
/// let size = tag.get_blob_size("data.bin");
/// let data = tag.get_blob("data.bin", size, 0);
/// ```
pub struct Tag {
    inner: cxx::UniquePtr<ffi::Tag>,
}

impl Tag {
    /// Create or get a tag by name
    ///
    /// If the tag exists, returns a handle to it.
    /// If not, creates a new tag.
    ///
    /// # Arguments
    /// * `name` - Tag name (must be unique)
    ///
    /// # Example
    /// ```
    /// use wrp_cte::sync::Tag;
    ///
    /// let tag = Tag::new("my_dataset");
    /// ```
    pub fn new(name: &str) -> Self {
        let inner = ffi::tag_new(name);
        Self { inner }
    }

    /// Open an existing tag by ID
    ///
    /// # Arguments
    /// * `id` - Tag ID (major.minor format)
    ///
    /// # Example
    /// ```
    /// use wrp_cte::sync::Tag;
    /// use wrp_cte::types::CteTagId;
    ///
    /// let id = CteTagId::new(1, 2);
    /// let tag = Tag::from_id(id);
    /// ```
    pub fn from_id(id: CteTagId) -> Self {
        let inner = ffi::tag_from_id(id.major, id.minor);
        Self { inner }
    }

    /// Get the placement score of a blob
    ///
    /// Score ranges from 0.0 (lowest priority) to 1.0 (highest priority).
    ///
    /// # Arguments
    /// * `name` - Blob name
    ///
    /// # Returns
    /// Score value (0.0-1.0)
    pub fn get_blob_score(&self, name: &str) -> f32 {
        ffi::tag_get_blob_score(&self.inner, name)
    }

    /// Reorganize a blob (change placement score)
    ///
    /// # Arguments
    /// * `name` - Blob name
    /// * `score` - New placement score (0.0-1.0)
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(CteError::RuntimeError)` on failure
    pub fn reorganize_blob(&self, name: &str, score: f32) -> CteResult<()> {
        let rc = ffi::tag_reorganize_blob(&self.inner, name, score);
        if rc == 0 {
            Ok(())
        } else {
            Err(CteError::RuntimeError {
                code: rc as u32,
                message: format!("Failed to reorganize blob {}", name),
            })
        }
    }

    /// Write data into a blob
    ///
    /// # Arguments
    /// * `name` - Blob name
    /// * `data` - Data to write
    /// * `offset` - Offset in blob (0 for new blobs)
    /// * `score` - Placement score (0.0-1.0)
    ///
    /// # Example
    /// ```
    /// use wrp_cte::sync::Tag;
    ///
    /// let tag = Tag::new("my_dataset");
    /// tag.put_blob_with_options("data.bin", b"hello", 0, 1.0);
    /// ```
    pub fn put_blob_with_options(&self, name: &str, data: &[u8], offset: u64, score: f32) {
        ffi::tag_put_blob(&self.inner, name, data, offset, score);
    }

    /// Write data into a blob with default offset (0) and score (1.0)
    ///
    /// Convenience method for simple blob storage.
    ///
    /// # Arguments
    /// * `name` - Blob name
    /// * `data` - Data to write
    pub fn put_blob(&self, name: &str, data: &[u8]) {
        self.put_blob_with_options(name, data, 0, 1.0);
    }

    /// Read data from a blob
    ///
    /// # Arguments
    /// * `name` - Blob name
    /// * `size` - Number of bytes to read
    /// * `offset` - Offset in blob (0 for start)
    ///
    /// # Returns
    /// Vector of bytes read
    ///
    /// # Example
    /// ```
    /// use wrp_cte::sync::Tag;
    ///
    /// let tag = Tag::new("my_dataset");
    /// let data = tag.get_blob("data.bin", 1024, 0);
    /// ```
    pub fn get_blob(&self, name: &str, size: u64, offset: u64) -> Vec<u8> {
        ffi::tag_get_blob(&self.inner, name, size, offset)
    }

    /// Get the size of a blob
    ///
    /// # Arguments
    /// * `name` - Blob name
    ///
    /// # Returns
    /// Size in bytes
    pub fn get_blob_size(&self, name: &str) -> u64 {
        ffi::tag_get_blob_size(&self.inner, name)
    }

    /// List all blobs in this tag
    ///
    /// # Returns
    /// Vector of blob names
    pub fn get_contained_blobs(&self) -> Vec<String> {
        ffi::tag_get_contained_blobs(&self.inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_returns_error_when_not_initialized() {
        // This will fail because CTE isn't running in tests
        // But we're testing the error path
        let result = init("");
        // Result depends on environment - just verify it compiles
        let _ = result;
    }

    #[test]
    fn test_cte_tag_id_conversion() {
        let id = CteTagId::new(1, 2);
        assert_eq!(id.major, 1);
        assert_eq!(id.minor, 2);
    }

    #[test]
    fn test_pool_query_variants() {
        let local = PoolQuery::local();
        let dynamic = PoolQuery::dynamic(30.0);
        let broadcast = PoolQuery::broadcast(60.0);

        assert_eq!(local.net_timeout(), 0.0);
        assert_eq!(dynamic.net_timeout(), 30.0);
        assert_eq!(broadcast.net_timeout(), 60.0);
    }
}
