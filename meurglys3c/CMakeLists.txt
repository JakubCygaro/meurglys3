cmake_minimum_required(VERSION 3.30)

set(testSources
    pack.c
    pack_inspect.c
    write_package.c
    package_methods.c
    insert_into_package.c
    remove_from_package.c
    list_files.c
)
set(meu3TestsDir ${CMAKE_CURRENT_SOURCE_DIR}/tests)

foreach(source IN LISTS testSources)
    set(testExe meu3_${source}_test)
    add_executable(${testExe}
        ${meu3TestsDir}/${source}
    )
    target_include_directories(${testExe}
        PRIVATE ${meu3Include}
    )
    target_link_directories(${testExe}
        PRIVATE ${meu3LinkDirectory}
    )
    target_link_libraries(${testExe}
        ${meu3StaticLib}
        ${meu3LibsToLink}
    )
    add_test(NAME meu3_${source}.test
        COMMAND ${testExe}
        WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}
    )
endforeach()

file(COPY ${CMAKE_CURRENT_SOURCE_DIR}/tests/test_dir DESTINATION .)
file(MAKE_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}/dump)
