// 文件路径: /Thanatos/daemon/cpp/src/shell_interface.cpp
#include "shell_interface.h"
#include <cstdio>
#include <iostream>
#include <memory>
#include <stdexcept>
#include <array>

std::string execute_shell_command(const std::string& command) {
    std::array<char, 256> buffer;
    std::string result;
    
    // Using popen to execute a command and read its output.
    // The '2>&1' part redirects stderr to stdout, so we capture both.
    std::string full_command = command + " 2>&1";
    
    std::unique_ptr<FILE, decltype(&pclose)> pipe(popen(full_command.c_str(), "r"), pclose);
    
    if (!pipe) {
        std::cerr << "popen() failed for command: " << command << std::endl;
        return "ERROR: popen() failed";
    }
    
    while (fgets(buffer.data(), buffer.size(), pipe.get()) != nullptr) {
        result += buffer.data();
    }
    
    return result;
}
