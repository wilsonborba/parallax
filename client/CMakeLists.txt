cmake_minimum_required(VERSION 3.16)
project(parallax_client LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_executable(parallax_client
    src/main.cpp
    src/OpenXRApp.cpp
    src/UdpReceiver.cpp
    src/MediaCodecDecoder.cpp
    src/QuadRenderer.cpp
)

target_include_directories(parallax_client PRIVATE include)

find_package(OpenXR QUIET)
if (OpenXR_FOUND)
    target_link_libraries(parallax_client PRIVATE OpenXR::openxr_loader)
    target_compile_definitions(parallax_client PRIVATE PARALLAX_HAVE_OPENXR=1)
else()
    target_compile_definitions(parallax_client PRIVATE PARALLAX_HAVE_OPENXR=0)
endif()
