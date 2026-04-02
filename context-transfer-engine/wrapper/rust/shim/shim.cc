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

#include "shim/shim.h"

#include <chimaera/bdev/bdev_client.h>
#include <wrp_cte/core/content_transfer_engine.h>

namespace cte_ffi {

// Thread-safe initialization globals
std::once_flag g_init_flag;
bool g_init_done = false;

// Initialization with thread safety
int32_t cte_init(rust::Str config_path) {
  std::call_once(g_init_flag, [&]() {
    std::string path(config_path.data(), config_path.size());
    bool ok = chi::CHIMAERA_INIT(chi::ChimaeraMode::kClient, true);
    if (!ok) {
      g_init_done = false;
      return;
    }
    g_init_done = wrp_cte::core::WRP_CTE_CLIENT_INIT(path);
  });
  return g_init_done ? 0 : -1;
}

// Client factory
std::unique_ptr<Client> client_new() { return std::make_unique<Client>(); }

// Poll telemetry log
std::vector<CteTelemetry> client_poll_telemetry(const Client& client,
                                                uint64_t min_time) {
  auto task = client.inner.AsyncPollTelemetryLog(min_time);
  task.Wait();

  std::vector<CteTelemetry> result;
  for (const auto& entry : task->entries_) {
    result.push_back(
        CteTelemetry{static_cast<uint32_t>(entry.op_), entry.off_, entry.size_,
                     CteTagId{entry.tag_id_.major_, entry.tag_id_.minor_},
                     SteadyTime{entry.mod_time_.time_since_epoch().count()},
                     SteadyTime{entry.read_time_.time_since_epoch().count()},
                     entry.logical_time_});
  }
  return result;
}

// Reorganize blob (returns error code)
int32_t client_reorganize_blob(const Client& client, uint32_t major,
                               uint32_t minor, rust::Str name, float score) {
  wrp_cte::core::TagId tag_id(major, minor);
  std::string blob_name(name.data(), name.size());
  auto task = client.inner.AsyncReorganizeBlob(tag_id, blob_name, score);
  task.Wait();
  return task->GetReturnCode();
}

// Delete blob (returns error code)
int32_t client_del_blob(const Client& client, uint32_t major, uint32_t minor,
                        rust::Str name) {
  wrp_cte::core::TagId tag_id(major, minor);
  std::string blob_name(name.data(), name.size());
  auto task = client.inner.AsyncDelBlob(tag_id, blob_name);
  task.Wait();
  return task->GetReturnCode();
}

// Pool query factory functions
std::unique_ptr<chi::PoolQuery> pool_query_broadcast(float timeout) {
  return std::make_unique<chi::PoolQuery>(chi::PoolQuery::Broadcast(timeout));
}

std::unique_ptr<chi::PoolQuery> pool_query_dynamic(float timeout) {
  return std::make_unique<chi::PoolQuery>(chi::PoolQuery::Dynamic(timeout));
}

std::unique_ptr<chi::PoolQuery> pool_query_local() {
  return std::make_unique<chi::PoolQuery>(chi::PoolQuery::Local());
}

// Tag factory functions
std::unique_ptr<Tag> tag_new(rust::Str name) {
  std::string n(name.data(), name.size());
  return std::make_unique<Tag>(n);
}

std::unique_ptr<Tag> tag_from_id(uint32_t major, uint32_t minor) {
  wrp_cte::core::TagId id(major, minor);
  return std::make_unique<Tag>(id);
}

// Tag operations
float tag_get_blob_score(const Tag& tag, rust::Str name) {
  std::string n(name.data(), name.size());
  return tag.inner.GetBlobScore(n);
}

int32_t tag_reorganize_blob(const Tag& tag, rust::Str name, float score) {
  std::string n(name.data(), name.size());
  tag.inner.ReorganizeBlob(n, score);
  // Tag methods don't return error codes directly
  return 0;
}

void tag_put_blob(const Tag& tag, rust::Str name,
                  rust::Slice<const uint8_t> data, uint64_t offset,
                  float score) {
  std::string n(name.data(), name.size());
  tag.inner.PutBlob(n, reinterpret_cast<const char*>(data.data()), data.size(),
                    static_cast<size_t>(offset), score);
}

std::vector<uint8_t> tag_get_blob(const Tag& tag, rust::Str name, uint64_t size,
                                  uint64_t offset) {
  std::string n(name.data(), name.size());
  auto buf = std::vector<uint8_t>(size);
  tag.inner.GetBlob(n, reinterpret_cast<char*>(buf.data()),
                    static_cast<size_t>(size), static_cast<size_t>(offset));
  return buf;
}

uint64_t tag_get_blob_size(const Tag& tag, rust::Str name) {
  std::string n(name.data(), name.size());
  return tag.inner.GetBlobSize(n);
}

std::vector<std::string> tag_get_contained_blobs(const Tag& tag) {
  return tag.inner.GetContainedBlobs();
}

}  // namespace cte_ffi