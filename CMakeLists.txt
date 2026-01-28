cmake_minimum_required(VERSION 3.30)
enable_testing()
project(Meurglys3)

set(allowedBuilds
    Debug
    Release
)
if(NOT ${CMAKE_CONFIGURATION_TYPES})
    if(NOT ${CMAKE_BUILD_TYPE})
        set(${CMAKE_BUILD_TYPE} Debug)
    endif()

    if(NOT ${CMAKE_BUILD_TYPE} IN_LIST allowedBuilds)
        message(FATAL_ERROR "Unknown build type ${CMAKE_BUILD_TYPE}")
    endif()
endif()

set(meu3Include
    ${CMAKE_CURRENT_SOURCE_DIR}/target/include
)
set(meu3LinkDirectory
    ${CMAKE_CURRENT_SOURCE_DIR}/target/release
    ${CMAKE_CURRENT_SOURCE_DIR}/target/debug
)
set(meu3StaticLibTargetDir "${CMAKE_SOURCE_DIR}/target/meu3_cmake")

# Build meurglys3c manually into a special directory
message(STATUS "Building meurglys3c library")
execute_process(COMMAND cargo  build -r --target-dir ${meu3StaticLibTargetDir}
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/meurglys3c
    RESULT_VARIABLE cargoMakeResult
    OUTPUT_VARIABLE cargoMakeOutput
    ERROR_VARIABLE cargoMakeError
)
if (cargoMakeResult)
    message(FATAL_ERROR "Failed to build the rust project")
    message(FATAL_ERROR "${cargoMakeError}")
endif()
message(STATUS "All crates successfully built")
# Find the static library
find_file(meu3StaticLib
    libmeurglys3c.a
    PATHS ${meu3StaticLibTargetDir}/release/
    REQUIRED
    NO_DEFAULT_PATH
)
# Get all required dynamic libraries for the linker
execute_process(
    COMMAND cargo rustc -- --print native-static-libs --crate-type staticlib -Awarnings
    WORKING_DIRECTORY ${CMAKE_CURRENT_LIST_DIR}/meurglys3c
    RESULT_VARIABLE rustcResult
    OUTPUT_VARIABLE rustcOutput
    ERROR_VARIABLE rustcOutput
    OUTPUT_STRIP_TRAILING_WHITESPACE
    ERROR_STRIP_TRAILING_WHITESPACE
)
if (rustcResult)
    message(FATAL_ERROR "Failed to build the rust project with cargo make")
    message(FATAL_ERROR "${rustcOutput}")
endif()

set(libsToLinkRegex "native-static-libs: \(.*\) +F" )
# output like so:
# note: native-static-libs: -lkernel32 -ladvapi32 -lntdll -luserenv -lws2_32 -lkernel32
string(REPLACE "\n" ";" split ${rustcOutput})
string(REGEX MATCH ${libsToLinkRegex} meu3LibsToLink ${split})
set(meu3LibsToLink ${CMAKE_MATCH_1})
string(REPLACE " " ";" meu3LibsToLink ${meu3LibsToLink})
list(FILTER meu3LibsToLink INCLUDE REGEX "[^ ]")

string(REPLACE ";" " " linkerFlags "${meu3LibsToLink}")

message(STATUS "To link: ${linkerFlags}")
configure_file(LinkerFlags.txt.in ${meu3StaticLibTargetDir}/LinkerFlags.txt @ONLY)

set(meu3LinkLibs TYPE STRING "${meu3LibsToLink}" CACHE INTERNAL "meu3LinkLibraries")
set(meu3StaticLibs "${meu3StaticLib}" CACHE INTERNAL "meu3StaticLib")
set(meu3IncludeDirs "${meu3Include}" CACHE INTERNAL "meu3Include")

if(MEU3_BUILD_TESTS)
    add_subdirectory(meurglys3c)
endif()
