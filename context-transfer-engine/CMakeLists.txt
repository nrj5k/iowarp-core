cmake_minimum_required(VERSION 3.20)
project(content_transfer_engine)

# Set root directory for this component
set(WRP_CTE_ROOT ${CMAKE_CURRENT_SOURCE_DIR})

# Note: WRP_CTE_ENABLE_COMPRESS is defined in root CMakeLists.txt
# It controls both compressor chimod building and HSHM compression support

# Python bindings are controlled by WRP_CORE_ENABLE_PYTHON in root CMakeLists.txt
# CTE_ENABLE_ASAN is set by root CMakeLists.txt

# C++ standard, compile commands, RPATH, ASAN, and output directories are all set by root CMakeLists.txt

# CTE_ENABLE_CMAKE_DOTENV is set by root CMakeLists.txt
# DOTENV handling is done by root CMakeLists.txt

# CMake modules and ChiMod utilities are now provided by IowarpCoreCommon.cmake
# All find_package calls are handled by root CMakeLists.txt
# Targets chimaera::cxx, chimaera_admin_client, etc. are already available

# Python bindings configuration
# nanobind is configured by root CMakeLists.txt when WRP_CORE_ENABLE_PYTHON=ON
# No local nanobind setup needed - use the root configuration
if(WRP_CORE_ENABLE_PYTHON)
  if(NOT COMMAND nanobind_add_module)
    message(WARNING "WRP_CORE_ENABLE_PYTHON is ON but nanobind_add_module command not found. Python bindings may fail.")
  else()
    message(STATUS "Python bindings available - nanobind configured by root CMakeLists.txt")
  endif()
endif()

message(STATUS "Found Chimaera framework - building full ChiMod implementation")

# Add the core ChiMod
add_subdirectory(core)

# Add the compressor ChiMod (optional - only if compression is enabled)
if(WRP_CTE_ENABLE_COMPRESS)
    message(STATUS "CTE: Compression enabled - building compressor chimod")
    add_subdirectory(compressor)
else()
    message(STATUS "CTE: Compression disabled - skipping compressor chimod (use -DWRP_CTE_ENABLE_COMPRESS=ON to enable)")
endif()

# Add the bdev ChiMod (when it exists)
# add_subdirectory(chimods/bdev)

# Add adapters
add_subdirectory(adapter)

# Add UVM (GPU virtual memory manager) -- requires CUDA
add_subdirectory(uvm)

# Enable testing
# CTest is included by root CMakeLists.txt
include(CTest)

# Add tests if enabled
# Tests are controlled by WRP_CORE_ENABLE_TESTS in root CMakeLists.txt
if(WRP_CORE_ENABLE_TESTS)
  add_subdirectory(test)
  message(STATUS "CTE tests enabled - added test subdirectory")
endif()

# Add wrapper components (includes Python bindings if enabled)
add_subdirectory(wrapper)

# Add benchmarks if enabled
# Benchmarks are controlled by WRP_CORE_ENABLE_BENCHMARKS
if(WRP_CORE_ENABLE_BENCHMARKS)
  add_subdirectory(benchmark)
  message(STATUS "CTE benchmarks enabled - added benchmark subdirectory")
endif()

message(STATUS "CTE Core and Bdev ChiMods configured for compilation")

if(WRP_CORE_ENABLE_JARVIS)
    jarvis_repo_add(${WRP_CTE_ROOT}/test/jarvis_iowarp)
endif()

#------------------------------------------------------------------------------
# Rust Bindings Configuration
#------------------------------------------------------------------------------
if(WRP_CORE_ENABLE_RUST)
    if(NOT COMMAND corrosion_import_crate)
        message(WARNING "WRP_CORE_ENABLE_RUST is ON but Corrosion not found. Ensure CMakeLists.txt root fetched Corrosion.")
    else()
        message(STATUS "Building CTE Rust bindings - Corrosion configured")
        
        # Export paths for build.rs
        # Include paths: hermes_shm, chimaera core, chimaera modules, cereal, and CTE headers
        get_filename_component(HSHM_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../context-transport-primitives" ABSOLUTE)
        get_filename_component(CHIMAERA_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../context-runtime" ABSOLUTE)
        get_filename_component(CTE_ROOT "${CMAKE_CURRENT_SOURCE_DIR}" ABSOLUTE)
        get_filename_component(CHIMODS_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../context-runtime/modules" ABSOLUTE)
        
        # Get cereal include directory from CMake target
        get_target_property(CEREAL_INCLUDE_DIR cereal::cereal INTERFACE_INCLUDE_DIRECTORIES)
        if(NOT CEREAL_INCLUDE_DIR)
            # Fallback: find cereal include from cereal_DIR
            get_filename_component(CEREAL_ROOT "${cereal_DIR}/../../.." ABSOLUTE)
            set(CEREAL_INCLUDE_DIR "${CEREAL_ROOT}/include")
        endif()
        
        message(STATUS "CTE Rust include paths:")
        message(STATUS "  HSHM_ROOT: ${HSHM_ROOT}/include")
        message(STATUS "  CHIMAERA_ROOT: ${CHIMAERA_ROOT}/include")
        message(STATUS "  CTE_ROOT: ${CTE_ROOT}/core/include")
        message(STATUS "  CHIMODS_ROOT: ${CHIMODS_ROOT}")
        message(STATUS "  CEREAL_INCLUDE: ${CEREAL_INCLUDE_DIR}")
        
        # Validate ZMQ was found before configuring Rust bindings
        if(WRP_CORE_ENABLE_ZMQ AND NOT ZMQ_LIBS)
            message(FATAL_ERROR "ZeroMQ not found but required for Rust bindings. Install with: sudo apt-get install libzmq3-dev")
        endif()

        # Import Rust crate
        # NOTE: This must come AFTER all add_subdirectory calls for C++ targets
        # to ensure all libraries (wrp_cte_core_client, etc.) are fully defined
        corrosion_import_crate(
            MANIFEST_PATH ${CMAKE_CURRENT_SOURCE_DIR}/wrapper/rust/Cargo.toml
            FEATURES async
        )
        
        # Set environment variables for build.rs AFTER target is created
        # Format: ENV_VAR=value (passed to cargo as environment variables)
        # Include all ChiMod directories to avoid missing headers
        corrosion_set_env_vars(wrp_cte 
            "IOWARP_INCLUDE_DIR=${HSHM_ROOT}/include"
            "IOWARP_EXTRA_INCLUDES=${CHIMAERA_ROOT}/include:${CTE_ROOT}/core/include:${CHIMODS_ROOT}/admin/include:${CHIMODS_ROOT}/bdev/include:${CEREAL_INCLUDE_DIR}"
            "IOWARP_LIB_DIR=${CMAKE_BINARY_DIR}/bin"
            "IOWARP_ZMQ_LIBS=${ZMQ_LIBS}"
            "IOWARP_ZMQ_LIB_DIRS=${ZMQ_LIB_DIRS}"
        )
        
        message(STATUS "CTE Rust bindings configured: wrp_cte (from package wrp-cte-rs)")
    endif()
endif()
