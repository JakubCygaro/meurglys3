cmake_minimum_required(VERSION 3.30)

set(testSources 
    pack.c
)
set(testsDir tests)

foreach(source IN LISTS testSources)
    set(testExe ${source}_test)
    add_executable(${testExe} 
        ${testsDir}/${source}
    )
    target_include_directories(${testExe}
        PRIVATE ${meu3Include}
    )
    target_link_directories(${testExe}
        PRIVATE ${meu3LinkDirectory}
    )
    target_link_libraries(${testExe}
        ${meu3StaticLib}
        ${libsToLink}
    )
    add_test(NAME ${source}.test
        COMMAND ${testExe}
        WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}
    )
endforeach()

file(COPY tests/test_dir DESTINATION .)
