# 文件路径: /Thanatos/daemon/CMakeLists.txt

cmake_minimum_required(VERSION 3.22)

project(thanatosd LANGUAGES CXX C)

# --- 1. 全局设置 ---
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

# --- 2. 编译 Rust 静态库 ---
# 我们使用 ExternalProject_Add 来调用 cargo build
# 这使得 CMake 能够驱动 Rust 的编译过程
include(ExternalProject)
ExternalProject_Add(
    rust_logic_lib # 项目名
    SOURCE_DIR ${CMAKE_CURRENT_SOURCE_DIR}/rust
    # 使用 cargo build --release 来构建
    # --target 指定了交叉编译目标
    # CMAKE_TOOLCHAIN_FILE 变量由 NDK 提供，它会设置好正确的环境变量
    CONFIGURE_COMMAND ""
    BUILD_COMMAND ${CMAKE_COMMAND} -E env \
                  CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=${CMAKE_CXX_COMPILER} \
                  cargo build --release --target aarch64-linux-android
    INSTALL_COMMAND ""
    # 定义构建产物
    BINARY_DIR ${CMAKE_CURRENT_SOURCE_DIR}/rust
    LOG_BUILD 1
)

# 定义 Rust 库的输出路径和头文件路径
set(RUST_LIBRARY_PATH ${CMAKE_CURRENT_SOURCE_DIR}/rust/target/aarch64-linux-android/release/libthanatos_logic.a)
set(RUST_BRIDGE_HEADER_PATH ${CMAKE_CURRENT_BINARY_DIR}/rust/target/cxxbridge/thanatos_logic_bridge/src/ffi.rs.h)

# 创建一个 IMPORTED 库目标，以便在 CMake 中引用 Rust 库
add_library(thanatos_logic STATIC IMPORTED)
set_property(TARGET thanatos_logic PROPERTY IMPORTED_LOCATION ${RUST_LIBRARY_PATH})
# 添加依赖，确保在链接前 Rust 库已经被编译
add_dependencies(thanatos_logic rust_logic_lib)

# --- 3. 配置 gRPC 和 Protobuf (使用 find_package) ---
# 为了简化，我们假设 gRPC 和 Protobuf 已经通过某种方式安装在系统或 NDK toolchain 中。
# 实际项目中，通常会使用 FetchContent 或 ExternalProject 来管理这些依赖。
# 为简单起见，我们暂时跳过这部分，下一阶段再详细实现。
# find_package(Protobuf REQUIRED)
# find_package(gRPC REQUIRED)

# --- 4. C++ 源文件 ---
# 创建一个空的 main.cpp 来测试链接
file(WRITE cpp/src/main.cpp "#include <iostream>\n#include \"${RUST_BRIDGE_HEADER_PATH}\"\n\nint main() {\n    std::cout << \"[C++] Starting thanatosd...\" << std::endl;\n    int init_code = rust_layer_init();\n    std::cout << \"[C++] Rust layer init code: \" << init_code << std::endl;\n    int result = rust_process_data(42);\n    std::cout << \"[C++] Result from Rust: \" << result << std::endl;\n    return 0;\n}\n")

# --- 5. 定义可执行文件 ---
add_executable(thanatosd cpp/src/main.cpp)

# --- 6. 链接库 ---
# 将 Rust 库链接到我们的 C++ 可执行文件
target_link_libraries(thanatosd PRIVATE thanatos_logic)

# 告诉 C++ 编译器去哪里找 Rust 生成的头文件
target_include_directories(thanatosd PRIVATE
    ${RUST_BRIDGE_HEADER_PATH}/../.. # 需要包含 cxx.h 的路径
    ${RUST_BRIDGE_HEADER_PATH}/..   # 包含 ffi.rs.h 的路径
)

# 链接 gRPC 和 Protobuf (下一阶段启用)
# target_link_libraries(thanatosd PRIVATE ${gRPC_LIBRARIES} ${Protobuf_LIBRARIES})