// 文件路径: /Thanatos/daemon/cpp/src/kernel_interface.cpp
#include "kernel_interface.h"
#include <fstream>
#include <iostream>
#include <string>
#include <vector>
#include <numeric>
#include <unistd.h>

void set_oom_score_adj(int pid, int score) {
    std::string path = "/proc/" + std::to_string(pid) + "/oom_score_adj";
    std::ofstream file(path);
    if (file.is_open()) {
        file << score;
        file.close();
    } else {
        // This can fail if the process is already gone, which is not a critical error.
        // std::cerr << "Failed to open " << path << std::endl;
    }
}

long long parse_mem_value(const std::string& line) {
    long long val = 0;
    sscanf(line.c_str(), "%*s %lld kB", &val);
    return val;
}

int64_t get_available_memory_kb() {
    std::ifstream meminfo("/proc/meminfo");
    std::string line;
    while (std::getline(meminfo, line)) {
        if (line.rfind("MemAvailable:", 0) == 0) {
            return parse_mem_value(line);
        }
    }
    return 0; // Fallback
}

int64_t get_total_memory_kb() {
    std::ifstream meminfo("/proc/meminfo");
    std::string line;
    while (std::getline(meminfo, line)) {
        if (line.rfind("MemTotal:", 0) == 0) {
            return parse_mem_value(line);
        }
    }
    return 0; // Fallback
}


// --- Simplified CPU Usage ---
struct CpuTimes {
    long long user = 0, nice = 0, system = 0, idle = 0, iowait = 0, irq = 0, softirq = 0;
};

CpuTimes read_cpu_times() {
    std::ifstream stat_file("/proc/stat");
    std::string line;
    CpuTimes times;
    if (std::getline(stat_file, line) && line.rfind("cpu", 0) == 0) {
        sscanf(line.c_str(), "cpu %lld %lld %lld %lld %lld %lld %lld",
               &times.user, &times.nice, &times.system, &times.idle,
               &times.iowait, &times.irq, &times.softirq);
    }
    return times;
}

float get_cpu_usage_percent() {
    static CpuTimes prev_times = {0,0,0,0,0,0,0};
    
    CpuTimes current_times = read_cpu_times();

    auto prev_idle = prev_times.idle + prev_times.iowait;
    auto current_idle = current_times.idle + current_times.iowait;

    auto prev_non_idle = prev_times.user + prev_times.nice + prev_times.system + prev_times.irq + prev_times.softirq;
    auto current_non_idle = current_times.user + current_times.nice + current_times.system + current_times.irq + current_times.softirq;

    auto prev_total = prev_idle + prev_non_idle;
    auto current_total = current_idle + current_non_idle;

    auto total_diff = current_total - prev_total;
    auto idle_diff = current_idle - prev_idle;

    prev_times = current_times;

    if (total_diff == 0) return 0.0f;

    return (float)(total_diff - idle_diff) * 100.0f / total_diff;
}
