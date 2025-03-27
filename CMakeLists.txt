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
    ${CMAKE_SOURCE_DIR}/target/include
)
set(meu3LinkDirectory
    ${CMAKE_SOURCE_DIR}/target/release
    ${CMAKE_SOURCE_DIR}/target/debug
)
add_subdirectory(meurglys3c)
# if(${CMAKE_BUILD_TYPE} STREQUAL "Release")
# endif()
add_custom_target(cargoMake ALL
    COMMAND "cargo make release"
)
