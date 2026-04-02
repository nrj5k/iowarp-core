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

#------------------------------------------------------------------------------
# Rust Bindings Configuration
#------------------------------------------------------------------------------
if(WRP_CORE_ENABLE_RUST)
    if(NOT COMMAND corrosion_import_crate)
        message(WARNING "WRP_CORE_ENABLE_RUST is ON but Corrosion not found. Ensure CMakeLists.txt root fetched Corrosion.")
    else()
        message(STATUS "Building CTE Rust bindings - Corrosion configured")
        
        # Export paths for build.rs
        # Use HSHM_ROOT from context-transport-primitives for include directory
        set_property(GLOBAL PROPERTY IOWARP_INCLUDE_DIR "${HSHM_ROOT}/include")
        set_property(GLOBAL PROPERTY IOWARP_LIB_DIR "${CMAKE_INSTALL_PREFIX}/lib")
        
        # Import Rust crate
        corrosion_import_crate(
            MANIFEST_PATH ${CMAKE_CURRENT_SOURCE_DIR}/wrapper/rust/Cargo.toml
            FEATURES async
        )
        
        # Dependency: CTE client must be built first
        add_dependencies(wrp-cte-rs wrp_cte_core_client)
        
        message(STATUS "CTE Rust bindings configured: wrp-cte-rs")
    endif()
endif()

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
