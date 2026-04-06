# Rust bindings are handled by parent CMakeLists.txt via Corrosion
# This file defines ANERIS_RUSTFLAGS variable for parent scope consumption

# Define paths for aneris-profiler build (relative to wrapper/rust directory)
get_filename_component(HSHM_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../../../context-transport-primitives" ABSOLUTE)
get_filename_component(CHIMAERA_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../../../context-runtime" ABSOLUTE)
get_filename_component(CTE_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../../.." ABSOLUTE)
get_filename_component(CTE_CORE_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../.." ABSOLUTE)
get_filename_component(CHIMODS_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../../../context-runtime/modules" ABSOLUTE)

# Get cereal include directory
get_target_property(CEREAL_INCLUDE_DIR cereal::cereal INTERFACE_INCLUDE_DIRECTORIES)
if(NOT CEREAL_INCLUDE_DIR)
    get_filename_component(CEREAL_ROOT "${cereal_DIR}/../../.." ABSOLUTE)
    set(CEREAL_INCLUDE_DIR "${CEREAL_ROOT}/include")
endif()

#------------------------------------------------------------------------------
# Aneris Profiler Build Configuration
#------------------------------------------------------------------------------
# Define RUSTFLAGS for aneris compile-time paths
set(ANERIS_RUSTFLAGS "--cfg aneris_build_dir=\\\"${CMAKE_BINARY_DIR}\\\" --cfg aneris_source_dir=\\\"${CMAKE_SOURCE_DIR}\\\"" CACHE STRING "RUSTFLAGS for aneris compile-time paths")

# Make it available to parent scope
set(ANERIS_RUSTFLAGS ${ANERIS_RUSTFLAGS} PARENT_SCOPE)

# Generate complete aneris_telemetry wrapper script from template
configure_file(
    ${CMAKE_CURRENT_LIST_DIR}/Aneris_telemetry.in
    ${CMAKE_BINARY_DIR}/bin/aneris_telemetry
    @ONLY
)

file(CHMOD ${CMAKE_BINARY_DIR}/bin/aneris_telemetry 
    PERMISSIONS OWNER_READ OWNER_WRITE OWNER_EXECUTE GROUP_READ GROUP_EXECUTE WORLD_READ WORLD_EXECUTE
)

message(STATUS "Generated aneris_telemetry wrapper: ${CMAKE_BINARY_DIR}/bin/aneris_telemetry")
