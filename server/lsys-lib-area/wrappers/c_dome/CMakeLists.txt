cmake_minimum_required(VERSION 3.16)
project(area_dome C)
set(CMAKE_C_STANDARD 11)
IF(NOT EXISTS ${PROJECT_SOURCE_DIR}/../../target/debug AND NOT EXISTS ${PROJECT_SOURCE_DIR}/../../target/release)
    MESSAGE(FATAL_ERROR "area db not build.")
ENDIF()
if (EXISTS ${PROJECT_SOURCE_DIR}/../../target/release)
    LINK_DIRECTORIES(${PROJECT_SOURCE_DIR}/../../target/release)
    INCLUDE_DIRECTORIES(${PROJECT_SOURCE_DIR}/../../target/release)
else()
    if (EXISTS ${PROJECT_SOURCE_DIR}/../../target/debug)
        LINK_DIRECTORIES(${PROJECT_SOURCE_DIR}/../../target/debug)
        INCLUDE_DIRECTORIES(${PROJECT_SOURCE_DIR}/../../target/debug)
    endif()
endif()
add_executable(area_dome main.c)
if (CMAKE_HOST_WIN32)
    target_link_libraries(area_dome lsys_lib_area.dll.dll)
else()
    target_link_libraries(area_dome lsys_lib_area)
endif ()

#cd wrappers/c_dome
#windows:add dll path to env
#Path=..\..\target\debug or Path=..\..\target\debug

####or####
#cd target\debug
#..\..\wrappers\c_dome\cmake-build-*\area_dome.exe
