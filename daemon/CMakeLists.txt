# 文件路径: /Thanatos/daemon/CMakeLists.txt
# 成功
cmake_minimum_required(VERSION 3.22)

project(thanatosd LANGUAGES CXX C)

# --- 1. 全局设置 ---
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

# --- 2. 依赖管理: gRPC, Protobuf, libev ---
include(FetchContent)

# Find libev installed on the system (e.g., via sudo apt install libev-dev)
find_package(LibEV REQUIRED)

# Fetch Protobuf from Git
FetchContent_Declare(
    protobuf
    GIT_REPOSITORY https://github.com/protocolbuffers/protobuf.git
    GIT_TAG        v21.12 # Use a stable version
)
set(protobuf_BUILD_TESTS OFF) # Don't build protobuf tests
FetchContent_MakeAvailable(protobuf)

# Fetch gRPC from Git
FetchContent_Declare(
    gRPC
    GIT_REPOSITORY https://github.com/grpc/grpc.git
    GIT_TAG        v1.49.2 # A version compatible with protobuf v21.x
)
set(gRPC_INSTALL OFF)
set(gRPC_BUILD_TESTS OFF)
set(gRPC_BUILD_CSHARP_EXT OFF)
set(gRPC_BUILD_GRPC_PHP_PLUGIN OFF)
FetchContent_MakeAvailable(gRPC)


# --- 3. 生成 Protobuf/gRPC C++ 代码 ---
get_target_property(PROTOC_EXE protobuf::protoc EXECUTABLE_LOCATION)
set(GRPC_CPP_PLUGIN_EXE $<TARGET_FILE:grpc_cpp_plugin>)

set(PROTO_FILES ${CMAKE_CURRENT_SOURCE_DIR}/../ipc/proto/thanatos.proto)
set(PROTO_INCLUDE_DIR ${CMAKE_CURRENT_SOURCE_DIR}/../ipc/proto)
set(PROTO_GEN_DIR ${CMAKE_CURRENT_BINARY_DIR}/gen/proto)

add_custom_command(
    OUTPUT ${PROTO_GEN_DIR}/thanatos.pb.cc ${PROTO_GEN_DIR}/thanatos.pb.h ${PROTO_GEN_DIR}/thanatos.grpc.pb.cc ${PROTO_GEN_DIR}/thanatos.grpc.pb.h
    COMMAND ${PROTOC_EXE}
        --grpc_out=${PROTO_GEN_DIR}
        --cpp_out=${PROTO_GEN_DIR}
        -I ${PROTO_INCLUDE_DIR}
        --plugin=protoc-gen-grpc=${GRPC_CPP_PLUGIN_EXE}
        ${PROTO_FILES}
    DEPENDS ${PROTO_FILES}
    COMMENT "Generating C++ sources from ${PROTO_FILES}"
)

add_library(thanatos_proto
    ${PROTO_GEN_DIR}/thanatos.pb.cc
    ${PROTO_GEN_DIR}/thanatos.grpc.pb.cc
)
target_include_directories(thanatos_proto PUBLIC ${PROTO_GEN_DIR})
target_link_libraries(thanatos_proto PUBLIC protobuf::libprotobuf gRPC::grpc)


# --- 4. 编译 Rust 静态库 ---
include(ExternalProject)
set(RUST_BUILD_DIR ${CMAKE_CURRENT_BINARY_DIR}/rust_build)
ExternalProject_Add(
    rust_logic_lib
    SOURCE_DIR ${CMAKE_CURRENT_SOURCE_DIR}/rust
    CARGO_TARGET_DIR ${RUST_BUILD_DIR}
    CONFIGURE_COMMAND ""
    BUILD_COMMAND ${CMAKE_COMMAND} -E env
                  CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=${CMAKE_CXX_COMPILER}
                  cargo build --release --target aarch64-linux-android
    INSTALL_COMMAND ""
    LOG_BUILD 1
)

set(RUST_LIBRARY_PATH ${RUST_BUILD_DIR}/aarch64-linux-android/release/libthanatos_logic.a)
set(RUST_BRIDGE_HEADER_PATH ${RUST_BUILD_DIR}/cxxbridge/thanatos_logic_bridge/src/ffi.rs.h)

add_library(thanatos_logic STATIC IMPORTED)
set_property(TARGET thanatos_logic PROPERTY IMPORTED_LOCATION ${RUST_LIBRARY_PATH})
add_dependencies(thanatos_logic rust_logic_lib)


# --- 5. 定义 thanatosd 可执行文件 ---
add_executable(thanatosd
    cpp/src/main.cpp
    cpp/src/grpc_server.cpp
    cpp/src/timer_manager.cpp
    cpp/src/kernel_interface.cpp
    cpp/src/shell_interface.cpp
)

target_link_libraries(thanatosd PRIVATE
    thanatos_proto
    thanatos_logic
    gRPC::grpc++
    protobuf::libprotobuf
    cxx           # Link against the cxx library from rust build
    LibEV::ev     # Link against libev
    -ldl          # Explicitly link dl for some gRPC dependencies
    -lpthread     # Explicitly link pthread
)

target_include_directories(thanatosd PRIVATE
    cpp/src
    # Include CXX generated headers
    ${RUST_BRIDGE_HEADER_PATH}/../.. # cxx.h
    ${RUST_BRIDGE_HEADER_PATH}/..   # ffi.rs.h
    # Include Protobuf generated headers
    ${PROTO_GEN_DIR}
    # Include libev headers
    ${LIBEV_INCLUDE_DIRS}
)
# Ensure proto code is generated before compiling C++
add_dependencies(thanatosd thanatos_proto)

# --- 6. 定义 (可选的) test_client 可执行文件 ---
add_executable(test_client
    cpp/src/test_client.cpp
)
target_link_libraries(test_client PRIVATE
    thanatos_proto
    gRPC::grpc++
    protobuf::libprotobuf
)
target_include_directories(test_client PRIVATE
    ${PROTO_GEN_DIR}
)
add_dependencies(test_client thanatos_proto)
