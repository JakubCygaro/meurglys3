cmake_minimum_required(VERSION 3.30)

set(testSources 
    tests/pack.c
)

foreach(source IN LISTS testSources)
    set(testExe ${source}_test)
    add_executable(${testExe} 
        source
    )
    target_include_directories(${testExe}
        ${meu3Include}
    )
    target_link_directories(${testExe}
        ${meu3LinkDirectory}
    )
    add_test(NAME ${source}.test
        COMMAND ${testExe}
    )
endforeach()
