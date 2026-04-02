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

#pragma once

#include <chimaera/chimaera.h>
#include <wrp_cte/core/core_client.h>

#include <cstdint>
#include <memory>
#include <mutex>
#include <string>
#include <vector>

#include "rust/cxx.h"

namespace cte_ffi {

// Forward declarations
struct Client;
struct Tag;

// Thread-safe initialization
extern std::once_flag g_init_flag;
extern bool g_init_done;

// Shared structs (MUST match Rust layout exactly)
// CteTagId matches chi::UniqueId (8 bytes: major + minor)
struct CteTagId {
  uint32_t major;
  uint32_t minor;
};

struct SteadyTime {
  int64_t nanos;
};

struct CteTelemetry {
  uint32_t op;
  uint64_t off;
  uint64_t size;
  CteTagId tag_id;
  SteadyTime mod_time;
  SteadyTime read_time;
  uint64_t logical_time;
};

// Opaque wrapper types
struct Client {
  mutable wrp_cte::core::Client inner;
};

struct Tag {
  mutable wrp_cte::core::Tag inner;

  explicit Tag(const std::string& name) : inner(name) {}
  explicit Tag(const wrp_cte::core::TagId& id) : inner(id) {}
};

// Initialization (returns error code: 0 = success, non-zero = failure)
int32_t cte_init(rust::Str config_path);

// Client operations
std::unique_ptr<Client> client_new();
std::vector<CteTelemetry> client_poll_telemetry(const Client& client,
                                                uint64_t min_time);
int32_t client_reorganize_blob(const Client& client, uint32_t major,
                               uint32_t minor, rust::Str name, float score);
int32_t client_del_blob(const Client& client, uint32_t major, uint32_t minor,
                        rust::Str name);

// Pool query factory functions
std::unique_ptr<chi::PoolQuery> pool_query_broadcast(float timeout);
std::unique_ptr<chi::PoolQuery> pool_query_dynamic(float timeout);
std::unique_ptr<chi::PoolQuery> pool_query_local();

// Tag operations
std::unique_ptr<Tag> tag_new(rust::Str name);
std::unique_ptr<Tag> tag_from_id(uint32_t major, uint32_t minor);
float tag_get_blob_score(const Tag& tag, rust::Str name);
int32_t tag_reorganize_blob(const Tag& tag, rust::Str name, float score);
void tag_put_blob(const Tag& tag, rust::Str name,
                  rust::Slice<const uint8_t> data, uint64_t offset,
                  float score);
std::vector<uint8_t> tag_get_blob(const Tag& tag, rust::Str name, uint64_t size,
                                  uint64_t offset);
uint64_t tag_get_blob_size(const Tag& tag, rust::Str name);
std::vector<std::string> tag_get_contained_blobs(const Tag& tag);

}  // namespace cte_ffi