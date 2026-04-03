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

//! Integration tests for CTE runtime
//!
//! These tests require a running CTE runtime. They are marked with `#[ignore]`
//! by default to avoid failures in CI environments without the runtime.
//!
//! # Running Integration Tests
//!
//! ## Async Tests (with tokio runtime)
//! ```bash
//! # Set environment variable to start embedded runtime
//! CHI_WITH_RUNTIME=1 cargo test --ignored --features async
//! ```
//!
//! ## Sync Tests (with embedded runtime)
//! ```bash
//! # Set environment variable to start embedded runtime
//! CHI_WITH_RUNTIME=1 cargo test --ignored
//! ```
//!
//! ## Alternative: Start runtime separately
//! ```bash
//! # Terminal 1: Start CTE runtime
//! wrp_cte --config /path/to/config.yaml
//!
//! # Terminal 2: Run tests
//! cargo test --ignored --features async
//! ```
//!
//! # Prerequisites
//! - CTE runtime installed and available on PATH
//! - Configuration file (optional, defaults to embedded config)
//! - Sufficient system resources for shared memory operations

#[cfg(test)]
mod sync_tests {
    use wrp_cte::sync::{init, Client, Tag};
    use wrp_cte::types::CteTagId;

    /// Test CTE initialization with default configuration
    ///
    /// This test verifies that CTE can be initialized with an empty
    /// configuration path (using defaults).
    #[test]
    #[ignore = "Requires running CTE runtime"]
    fn test_init_default_config() {
        let result = init("");
        assert!(result.is_ok(), "CTE initialization should succeed with default config");
    }

    /// Test client creation
    ///
    /// Verifies that a CTE client can be created after initialization.
    #[test]
    #[ignore = "Requires running CTE runtime"]
    fn test_client_creation() {
        init("").expect("CTE initialization failed");
        let client = Client::new();
        assert!(client.is_ok(), "Client creation should succeed after init");
    }

    /// Test tag creation by name
    ///
    /// Creates a tag with a unique name and verifies it succeeds.
    #[test]
    #[ignore = "Requires running CTE runtime"]
    fn test_tag_creation_by_name() {
        init("").expect("CTE initialization failed");
        let tag_name = format!("test_tag_{}", std::process::id());
        let tag = Tag::new(&tag_name);
        let id = tag.id();
        assert!(id.major > 0 || id.minor > 0, "Tag ID should be valid");
    }

    /// Test tag creation by ID
    ///
    /// Verifies that an existing tag can be opened by its ID.
    #[test]
    #[ignore = "Requires running CTE runtime"]
    fn test_tag_creation_by_id() {
        init("").expect("CTE initialization failed");
        
        // First create a tag by name
        let tag_name = format!("test_tag_by_id_{}", std::process::id());
        let tag = Tag::new(&tag_name);
        let id = tag.id();
        
        // Then open it by ID
        let tag_by_id = Tag::from_id(id);
        let id_again = tag_by_id.id();
        
        assert_eq!(id.major, id_again.major, "Tag major IDs should match");
        assert_eq!(id.minor, id_again.minor, "Tag minor IDs should match");
    }

    /// Test blob put and get operations
    ///
    /// Writes data to a blob and reads it back, verifying integrity.
    #[test]
    #[ignore = "Requires running CTE runtime"]
    fn test_blob_put_get() {
        init("").expect("CTE initialization failed");
        
        let tag_name = format!("test_tag_blob_{}", std::process::id());
        let tag = Tag::new(&tag_name);
        
        // Write test data
        let blob_name = "test_blob.bin";
        let test_data = b"Hello, CTE!";
        tag.put_blob_with_options(blob_name, test_data, 0, 1.0)
            .expect("Blob put should succeed");
        
        // Get blob size
        let size = tag.get_blob_size(blob_name).expect("Get blob size should succeed");
        assert_eq!(size, test_data.len() as u64, "Blob size should match written data");
        
        // Read back data
        let read_data = tag.get_blob(blob_name, size, 0).expect("Blob get should succeed");
        assert_eq!(read_data, test_data, "Read data should match written data");
    }

    /// Test blob score operations
    ///
    /// Verifies that blob placement scores can be set and retrieved.
    #[test]
    #[ignore = "Requires running CTE runtime"]
    fn test_blob_score() {
        init("").expect("CTE initialization failed");
        
        let tag_name = format!("test_tag_score_{}", std::process::id());
        let tag = Tag::new(&tag_name);
        
        // Create a blob with score 1.0
        let blob_name = "scored_blob.bin";
        tag.put_blob_with_options(blob_name, b"test", 0, 1.0)
            .expect("Blob put should succeed");
        
        // Verify default score
        let score = tag.get_blob_score(blob_name).expect("Get blob score should succeed");
        assert!((score - 1.0).abs() < 0.01, "Default score should be 1.0");
        
        // Change the score
        tag.reorganize_blob(blob_name, 0.5).expect("Reorganize blob should succeed");
        
        // Verify new score
        let new_score = tag.get_blob_score(blob_name).expect("Get blob score should succeed");
        assert!((new_score - 0.5).abs() < 0.01, "New score should be 0.5");
    }

    /// Test blob deletion
    ///
    /// Verifies that blobs can be deleted from a tag.
    #[test]
    #[ignore = "Requires running CTE runtime"]
    fn test_blob_deletion() {
        init("").expect("CTE initialization failed");
        
        let tag_name = format!("test_tag_delete_{}", std::process::id());
        let tag = Tag::new(&tag_name);
        
        // Create a blob
        let blob_name = "deletable_blob.bin";
        tag.put_blob_with_options(blob_name, b"test data", 0, 1.0)
            .expect("Blob put should succeed");
        
        // Verify it exists
        let size = tag.get_blob_size(blob_name).expect("Get blob size should succeed");
        assert!(size > 0, "Blob should exist before deletion");
        
        // Delete the blob
        let tag_id = tag.id();
        let client = Client::new().expect("Client creation should succeed");
        client.del_blob(tag_id, blob_name).expect("Blob deletion should succeed");
        
        // Verify it's gone (should fail to get size)
        let result = tag.get_blob_size(blob_name);
        assert!(result.is_err(), "Blob should not exist after deletion");
    }

    /// Test telemetry polling
    ///
    /// Verifies that telemetry data can be retrieved from the client.
    #[test]
    #[ignore = "Requires running CTE runtime"]
    fn test_telemetry_polling() {
        init("").expect("CTE initialization failed");
        
        let client = Client::new().expect("Client creation should succeed");
        
        // Perform some operations to generate telemetry
        let tag_name = format!("test_tag_telemetry_{}", std::process::id());
        let tag = Tag::new(&tag_name);
        tag.put_blob_with_options("telemetry_blob.bin", b"test", 0, 1.0)
            .expect("Blob put should succeed");
        
        // Poll telemetry
        let telemetry = client.poll_telemetry(0).expect("Telemetry polling should succeed");
        
        // At minimum, we should have some entries after operations
        // Note: This may be empty if telemetry was already cleared
        // Just verify it doesn't panic
        for entry in telemetry {
            println!("Telemetry: op={:?}, size={}, tag={}.{}", 
                entry.op, entry.size, entry.tag_id.major, entry.tag_id.minor);
        }
    }

    /// Test client-level blob reorganization
    ///
    /// Verifies that blob scores can be changed via the client API.
    #[test]
    #[ignore = "Requires running CTE runtime"]
    fn test_client_reorganization() {
        init("").expect("CTE initialization failed");
        
        let tag_name = format!("test_tag_reorg_{}", std::process::id());
        let tag = Tag::new(&tag_name);
        
        // Create a blob
        let blob_name = "reorg_blob.bin";
        tag.put_blob_with_options(blob_name, b"test", 0, 1.0)
            .expect("Blob put should succeed");
        
        // Reorganize via client
        let tag_id = tag.id();
        let client = Client::new().expect("Client creation should succeed");
        client.reorganize_blob(tag_id, blob_name, 0.25)
            .expect("Client reorganization should succeed");
        
        // Verify new score
        let new_score = tag.get_blob_score(blob_name).expect("Get blob score should succeed");
        assert!((new_score - 0.25).abs() < 0.01, "New score should be 0.25");
    }

    /// Test multiple blobs in a tag
    ///
    /// Verifies that multiple blobs can be stored and listed in a tag.
    #[test]
    #[ignore = "Requires running CTE runtime"]
    fn test_multiple_blobs() {
        init("").expect("CTE initialization failed");
        
        let tag_name = format!("test_tag_multi_{}", std::process::id());
        let tag = Tag::new(&tag_name);
        
        // Create multiple blobs
        for i in 0..5 {
            let blob_name = format!("multi_blob_{}.bin", i);
            let data = format!("data_{}", i);
            tag.put_blob_with_options(&blob_name, data.as_bytes(), 0, 1.0)
                .expect("Blob put should succeed");
        }
        
        // List all blobs
        let blobs = tag.get_contained_blobs();
        assert!(blobs.len() >= 5, "Should have at least 5 blobs in tag");
    }

    /// Test blob write with offset
    ///
    /// Verifies that data can be written at specific offsets within a blob.
    #[test]
    #[ignore = "Requires running CTE runtime"]
    fn test_blob_write_with_offset() {
        init("").expect("CTE initialization failed");
        
        let tag_name = format!("test_tag_offset_{}", std::process::id());
        let tag = Tag::new(&tag_name);
        
        // Write initial data
        let blob_name = "offset_blob.bin";
        tag.put_blob_with_options(blob_name, b"Hello", 0, 1.0)
            .expect("Initial write should succeed");
        
        // Write at offset
        tag.put_blob_with_options(blob_name, b" World", 5, 1.0)
            .expect("Offset write should succeed");
        
        // Read full data
        let size = tag.get_blob_size(blob_name).expect("Get size should succeed");
        let data = tag.get_blob(blob_name, size, 0).expect("Read should succeed");
        
        assert_eq!(&data, b"Hello World", "Data should match combined writes");
    }
}

#[cfg(all(test, feature = "async"))]
mod async_tests {
    use wrp_cte::r#async::{Client, Tag};
    use wrp_cte::sync::init;

    /// Test async CTE initialization
    #[tokio::test]
    #[ignore = "Requires running CTE runtime"]
    async fn test_async_init_default_config() {
        let result = init("");
        // Note: async init is a re-export of sync init, so result handling may differ
        // depending on whether runtime is already initialized
        let _ = result;
    }

    /// Test async client creation
    #[tokio::test]
    #[ignore = "Requires running CTE runtime"]
    async fn test_async_client_creation() {
        init("").expect("CTE initialization failed");
        let client = Client::new().await;
        assert!(client.is_ok(), "Async client creation should succeed");
    }

    /// Test async tag creation by name
    #[tokio::test]
    #[ignore = "Requires running CTE runtime"]
    async fn test_async_tag_creation_by_name() {
        init("").expect("CTE initialization failed");
        
        let tag_name = format!("async_test_tag_{}", std::process::id());
        let tag = Tag::new(&tag_name).await.expect("Async tag creation should succeed");
        
        let id = tag.get_id().await.expect("Get tag ID should succeed");
        assert!(id.major > 0 || id.minor > 0, "Tag ID should be valid");
    }

    /// Test async tag creation by ID
    #[tokio::test]
    #[ignore = "Requires running CTE runtime"]
    async fn test_async_tag_creation_by_id() {
        init("").expect("CTE initialization failed");
        
        // First create a tag by name
        let tag_name = format!("async_test_tag_by_id_{}", std::process::id());
        let tag = Tag::new(&tag_name).await.expect("Async tag creation should succeed");
        let id = tag.get_id().await.expect("Get tag ID should succeed");
        
        // Then open it by ID
        let tag_by_id = Tag::from_id(id).await.expect("Async tag from ID should succeed");
        let id_again = tag_by_id.get_id().await.expect("Get tag ID should succeed");
        
        assert_eq!(id.major, id_again.major, "Tag major IDs should match");
        assert_eq!(id.minor, id_again.minor, "Tag minor IDs should match");
    }

    /// Test async blob put and get operations
    #[tokio::test]
    #[ignore = "Requires running CTE runtime"]
    async fn test_async_blob_put_get() {
        init("").expect("CTE initialization failed");
        
        let tag_name = format!("async_test_tag_blob_{}", std::process::id());
        let tag = Tag::new(&tag_name).await.expect("Async tag creation should succeed");
        
        // Write test data
        let blob_name = "async_test_blob.bin".to_string();
        let test_data = b"Hello, async CTE!".to_vec();
        tag.put_blob(blob_name.clone(), test_data.clone(), 0, 1.0)
            .await
            .expect("Async blob put should succeed");
        
        // Get blob size
        let size = tag.get_blob_size(&blob_name)
            .await
            .expect("Async get blob size should succeed");
        assert_eq!(size, test_data.len() as u64, "Blob size should match written data");
        
        // Read back data
        let read_data = tag.get_blob(blob_name.clone(), size, 0)
            .await
            .expect("Async blob get should succeed");
        assert_eq!(read_data, test_data, "Read data should match written data");
    }

    /// Test async blob score operations
    #[tokio::test]
    #[ignore = "Requires running CTE runtime"]
    async fn test_async_blob_score() {
        init("").expect("CTE initialization failed");
        
        let tag_name = format!("async_test_tag_score_{}", std::process::id());
        let tag = Tag::new(&tag_name).await.expect("Async tag creation should succeed");
        
        // Create a blob with score 1.0
        let blob_name = "async_scored_blob.bin".to_string();
        tag.put_blob(blob_name.clone(), b"test".to_vec(), 0, 1.0)
            .await
            .expect("Async blob put should succeed");
        
        // Verify default score
        let score = tag.get_blob_score(&blob_name)
            .await
            .expect("Async get blob score should succeed");
        assert!((score - 1.0).abs() < 0.01, "Default score should be 1.0");
        
        // Change the score
        tag.reorganize_blob(blob_name.clone(), 0.5)
            .await
            .expect("Async reorganize blob should succeed");
        
        // Verify new score
        let new_score = tag.get_blob_score(&blob_name)
            .await
            .expect("Async get blob score should succeed");
        assert!((new_score - 0.5).abs() < 0.01, "New score should be 0.5");
    }

    /// Test async blob deletion
    #[tokio::test]
    #[ignore = "Requires running CTE runtime"]
    async fn test_async_blob_deletion() {
        init("").expect("CTE initialization failed");
        
        let tag_name = format!("async_test_tag_delete_{}", std::process::id());
        let tag = Tag::new(&tag_name).await.expect("Async tag creation should succeed");
        
        // Create a blob
        let blob_name = "async_deletable_blob.bin".to_string();
        tag.put_blob(blob_name.clone(), b"test data".to_vec(), 0, 1.0)
            .await
            .expect("Async blob put should succeed");
        
        // Verify it exists
        let size = tag.get_blob_size(&blob_name)
            .await
            .expect("Async get blob size should succeed");
        assert!(size > 0, "Blob should exist before deletion");
        
        // Delete the blob
        let tag_id = tag.get_id().await.expect("Get tag ID should succeed");
        let client = Client::new().await.expect("Async client creation should succeed");
        client.del_blob(tag_id, blob_name.clone())
            .await
            .expect("Async blob deletion should succeed");
        
        // Verify it's gone
        let result = tag.get_blob_size(&blob_name).await;
        assert!(result.is_err(), "Blob should not exist after deletion");
    }

    /// Test async telemetry polling
    #[tokio::test]
    #[ignore = "Requires running CTE runtime"]
    async fn test_async_telemetry_polling() {
        init("").expect("CTE initialization failed");
        
        let client = Client::new().await.expect("Async client creation should succeed");
        
        // Perform some operations to generate telemetry
        let tag_name = format!("async_test_tag_telemetry_{}", std::process::id());
        let tag = Tag::new(&tag_name).await.expect("Async tag creation should succeed");
        tag.put_blob("async_telemetry_blob.bin".to_string(), b"test".to_vec(), 0, 1.0)
            .await
            .expect("Async blob put should succeed");
        
        // Poll telemetry
        let telemetry = client.poll_telemetry(0)
            .await
            .expect("Async telemetry polling should succeed");
        
        for entry in telemetry {
            println!("Async Telemetry: op={:?}, size={}, tag={}.{}", 
                entry.op, entry.size, entry.tag_id.major, entry.tag_id.minor);
        }
    }

    /// Test async client-level blob reorganization
    #[tokio::test]
    #[ignore = "Requires running CTE runtime"]
    async fn test_async_client_reorganization() {
        init("").expect("CTE initialization failed");
        
        let tag_name = format!("async_test_tag_reorg_{}", std::process::id());
        let tag = Tag::new(&tag_name).await.expect("Async tag creation should succeed");
        
        // Create a blob
        let blob_name = "async_reorg_blob.bin".to_string();
        tag.put_blob(blob_name.clone(), b"test".to_vec(), 0, 1.0)
            .await
            .expect("Async blob put should succeed");
        
        // Reorganize via client
        let tag_id = tag.get_id().await.expect("Get tag ID should succeed");
        let client = Client::new().await.expect("Async client creation should succeed");
        client.reorganize_blob(tag_id, blob_name.clone(), 0.25)
            .await
            .expect("Async client reorganization should succeed");
        
        // Verify new score
        let new_score = tag.get_blob_score(&blob_name)
            .await
            .expect("Async get blob score should succeed");
        assert!((new_score - 0.25).abs() < 0.01, "New score should be 0.25");
    }

    /// Test async multiple blobs in a tag
    #[tokio::test]
    #[ignore = "Requires running CTE runtime"]
    async fn test_async_multiple_blobs() {
        init("").expect("CTE initialization failed");
        
        let tag_name = format!("async_test_tag_multi_{}", std::process::id());
        let tag = Tag::new(&tag_name).await.expect("Async tag creation should succeed");
        
        // Create multiple blobs
        for i in 0..5 {
            let blob_name = format!("async_multi_blob_{}.bin", i);
            let data = format!("data_{}", i);
            tag.put_blob(blob_name, data.into_bytes(), 0, 1.0)
                .await
                .expect("Async blob put should succeed");
        }
        
        // List all blobs
        let blobs = tag.get_contained_blobs()
            .await
            .expect("Async get contained blobs should succeed");
        assert!(blobs.len() >= 5, "Should have at least 5 blobs in tag");
    }

    /// Test async blob write with offset
    #[tokio::test]
    #[ignore = "Requires running CTE runtime"]
    async fn test_async_blob_write_with_offset() {
        init("").expect("CTE initialization failed");
        
        let tag_name = format!("async_test_tag_offset_{}", std::process::id());
        let tag = Tag::new(&tag_name).await.expect("Async tag creation should succeed");
        
        // Write initial data
        let blob_name = "async_offset_blob.bin".to_string();
        tag.put_blob(blob_name.clone(), b"Hello".to_vec(), 0, 1.0)
            .await
            .expect("Async initial write should succeed");
        
        // Write at offset
        tag.put_blob(blob_name.clone(), b" World".to_vec(), 5, 1.0)
            .await
            .expect("Async offset write should succeed");
        
        // Read full data
        let size = tag.get_blob_size(&blob_name)
            .await
            .expect("Async get size should succeed");
        let data = tag.get_blob(blob_name.clone(), size, 0)
            .await
            .expect("Async read should succeed");
        
        assert_eq!(&data, b"Hello World", "Data should match combined writes");
    }
}