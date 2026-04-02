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

//! Asynchronous CTE API (default feature)
//!
//! This module provides async/await wrappers around the blocking CTE FFI.
//! Uses `tokio::task::spawn_blocking` to bridge C++ blocking calls.
//!
//! # Example
//! ```no_run
//! use wrp_cte::{Client, Tag};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize and create client
//!     let client = Client::new().await?;
//!
//!     // Create or open a tag
//!     let tag = Tag::new("my_dataset").await?;
//!
//!     // Store data asynchronously
//!     tag.put_blob("data.bin".to_string(), b"hello".to_vec(), 0, 1.0).await;
//!
//!     // Get telemetry
//!     let telemetry = client.poll_telemetry(0).await?;
//!     for entry in telemetry {
//!         println!("Op: {:?}, Size: {}", entry.op, entry.size);
//!     }
//!
//!     Ok(())
//! }
//! ```

pub use crate::sync::{init, BdevType, ChimaeraMode, SteadyTime};
pub use crate::types::{CteOp, CteTagId, CteTelemetry, PoolQuery};

use crate::error::{CteError, CteResult};
use crate::ffi;

/// Async CTE client
///
/// Provides async methods for client-level operations.
/// Uses spawn_blocking to bridge C++ blocking calls.
pub struct Client {
    _marker: std::marker::PhantomData<()>,
}

impl Client {
    /// Create a new CTE client
    ///
    /// # Returns
    /// * `Ok(Client)` on success
    /// * `Err(CteError::InitFailed)` if initialization fails
    ///
    /// # Example
    /// ```no_run
    /// use wrp_cte::Client;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::new().await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn new() -> CteResult<Self> {
        // Initialize is sync (must complete before any operations)
        crate::sync::init("")?;
        Ok(Self {
            _marker: std::marker::PhantomData,
        })
    }

    /// Poll telemetry log from CTE
    ///
    /// # Arguments
    /// * `min_time` - Minimum timestamp to fetch (0 for all)
    ///
    /// # Returns
    /// Vector of telemetry entries
    pub async fn poll_telemetry(&self, min_time: u64) -> CteResult<Vec<CteTelemetry>> {
        tokio::task::spawn_blocking(move || {
            let client = ffi::client_new();
            let raw = ffi::client_poll_telemetry(&client, min_time);
            raw.into_iter()
                .map(|t| CteTelemetry {
                    op: t.op.into(),
                    off: t.off,
                    size: t.size,
                    tag_id: (&t.tag_id).into(),
                    mod_time: (&t.mod_time).into(),
                    read_time: (&t.read_time).into(),
                    logical_time: t.logical_time,
                })
                .collect::<Vec<_>>()
        })
        .await
        .map_err(|e| CteError::FfiError {
            message: e.to_string(),
        })
    }

    /// Reorganize a blob (change placement score)
    ///
    /// # Arguments
    /// * `tag_id` - ID of the tag containing the blob
    /// * `name` - Blob name
    /// * `score` - New placement score (0.0-1.0)
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(CteError::RuntimeError)` on failure
    pub async fn reorganize_blob(
        &self,
        tag_id: CteTagId,
        name: String,
        score: f32,
    ) -> CteResult<()> {
        tokio::task::spawn_blocking(move || {
            let client = ffi::client_new();
            let rc = ffi::client_reorganize_blob(
                &client,
                tag_id.major,
                tag_id.minor,
                &name,
                score,
            );
            if rc == 0 {
                Ok(())
            } else {
                Err(CteError::RuntimeError {
                    code: rc as u32,
                    message: format!("Failed to reorganize blob {}", name),
                })
            }
        })
        .await
        .map_err(|e| CteError::FfiError {
            message: e.to_string(),
        })?
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
    pub async fn del_blob(&self, tag_id: CteTagId, name: String) -> CteResult<()> {
        tokio::task::spawn_blocking(move || {
            let client = ffi::client_new();
            let rc = ffi::client_del_blob(
                &client,
                tag_id.major,
                tag_id.minor,
                &name,
            );
            if rc == 0 {
                Ok(())
            } else {
                Err(CteError::RuntimeError {
                    code: rc as u32,
                    message: format!("Failed to delete blob {}", name),
                })
            }
        })
        .await
        .map_err(|e| CteError::FfiError {
            message: e.to_string(),
        })?
    }
}

/// Async tag wrapper
///
/// Provides async methods for tag/blob operations.
/// Uses spawn_blocking to bridge C++ blocking calls.
pub struct Tag {
    inner: cxx::UniquePtr<ffi::Tag>,
}

impl Tag {
    /// Create or get a tag by name
    ///
    /// # Arguments
    /// * `name` - Tag name
    pub async fn new(name: &str) -> CteResult<Self> {
        let name = name.to_string();
        let inner = tokio::task::spawn_blocking(move || ffi::tag_new(&name))
            .await
            .map_err(|e| CteError::FfiError {
                message: e.to_string(),
            })?;
        Ok(Self { inner })
    }

    /// Open an existing tag by ID
    ///
    /// # Arguments
    /// * `id` - Tag ID
    pub async fn from_id(id: CteTagId) -> CteResult<Self> {
        let inner = tokio::task::spawn_blocking(move || {
            ffi::tag_from_id(id.major, id.minor)
        })
        .await
        .map_err(|e| CteError::FfiError {
            message: e.to_string(),
        })?;
        Ok(Self { inner })
    }

    /// Get the placement score of a blob
    ///
    /// # Arguments
    /// * `name` - Blob name
    ///
    /// # Returns
    /// Score value (0.0-1.0)
    ///
    /// # Panics
    /// This method is not yet implemented. Async Tag operations require
    /// `Arc<Mutex<Tag>>` for thread-safe access to the underlying C++ object.
    /// Use the sync API (`wrp_cte::sync::Tag`) instead.
    pub async fn get_blob_score(&self, _name: &str) -> f32 {
        panic!(
            "get_blob_score is not implemented for async API. \
             Async Tag operations require Arc<Mutex<Tag>> for thread-safe access \
             to the underlying C++ object. Use the sync API (wrp_cte::sync::Tag) instead."
        )
    }

    /// Reorganize a blob (change placement score)
    ///
    /// # Panics
    /// This method is not yet implemented. Async Tag operations require
    /// `Arc<Mutex<Tag>>` for thread-safe access to the underlying C++ object.
    /// Use the sync API (`wrp_cte::sync::Tag`) instead.
    pub async fn reorganize_blob(&self, _name: String, _score: f32) -> CteResult<()> {
        Err(CteError::NotImplemented {
            feature: "reorganize_blob".to_string(),
            reason: "Async Tag operations require Arc<Mutex<Tag>> for thread-safe access \
                     to the underlying C++ object. Use the sync API (wrp_cte::sync::Tag) instead."
                .to_string(),
        })
    }

    /// Write data into a blob
    ///
    /// # Panics
    /// This method is not yet implemented. Async Tag operations require
    /// `Arc<Mutex<Tag>>` for thread-safe access to the underlying C++ object.
    /// Use the sync API (`wrp_cte::sync::Tag`) instead.
    pub async fn put_blob(&self, _name: String, _data: Vec<u8>, _offset: u64, _score: f32) {
        panic!(
            "put_blob is not implemented for async API. \
             Async Tag operations require Arc<Mutex<Tag>> for thread-safe access \
             to the underlying C++ object. Use the sync API (wrp_cte::sync::Tag) instead."
        )
    }

    /// Read data from a blob
    ///
    /// # Panics
    /// This method is not yet implemented. Async Tag operations require
    /// `Arc<Mutex<Tag>>` for thread-safe access to the underlying C++ object.
    /// Use the sync API (`wrp_cte::sync::Tag`) instead.
    pub async fn get_blob(&self, _name: String, _size: u64, _offset: u64) -> Vec<u8> {
        panic!(
            "get_blob is not implemented for async API. \
             Async Tag operations require Arc<Mutex<Tag>> for thread-safe access \
             to the underlying C++ object. Use the sync API (wrp_cte::sync::Tag) instead."
        )
    }

    /// Get the size of a blob
    ///
    /// # Panics
    /// This method is not yet implemented. Async Tag operations require
    /// `Arc<Mutex<Tag>>` for thread-safe access to the underlying C++ object.
    /// Use the sync API (`wrp_cte::sync::Tag`) instead.
    pub async fn get_blob_size(&self, _name: &str) -> u64 {
        panic!(
            "get_blob_size is not implemented for async API. \
             Async Tag operations require Arc<Mutex<Tag>> for thread-safe access \
             to the underlying C++ object. Use the sync API (wrp_cte::sync::Tag) instead."
        )
    }

    /// List all blobs in this tag
    ///
    /// # Panics
    /// This method is not yet implemented. Async Tag operations require
    /// `Arc<Mutex<Tag>>` for thread-safe access to the underlying C++ object.
    /// Use the sync API (`wrp_cte::sync::Tag`) instead.
    pub async fn get_contained_blobs(&self) -> Vec<String> {
        panic!(
            "get_contained_blobs is not implemented for async API. \
             Async Tag operations require Arc<Mutex<Tag>> for thread-safe access \
             to the underlying C++ object. Use the sync API (wrp_cte::sync::Tag) instead."
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_query_variants() {
        let local = PoolQuery::local();
        let dynamic = PoolQuery::dynamic(30.0);

        assert_eq!(local.net_timeout(), 0.0);
        assert_eq!(dynamic.net_timeout(), 30.0);
    }
}