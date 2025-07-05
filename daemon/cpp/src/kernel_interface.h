// 文件路径: /Thanatos/daemon/cpp/src/kernel_interface.h
#ifndef THANATOS_KERNEL_INTERFACE_H
#define THANATOS_KERNEL_INTERFACE_H

#include <cstdint>
#include <string>

void set_oom_score_adj(int pid, int score);
int64_t get_available_memory_kb();
int64_t get_total_memory_kb();
float get_cpu_usage_percent(); // Note: This is a simplified implementation

#endif //THANATOS_KERNEL_INTERFACE_H
