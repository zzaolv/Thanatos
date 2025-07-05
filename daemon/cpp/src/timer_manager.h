// 文件路径: /Thanatos/daemon/cpp/src/timer_manager.h
#ifndef THANATOS_TIMER_MANAGER_H
#define THANATOS_TIMER_MANAGER_H

#include <cstdint>

// Forward declaration of libev's loop struct
struct ev_loop;

// Initializes the timer manager with the main event loop.
void init_timer_manager(struct ev_loop* loop);

// Creates a one-shot timer that fires after a specified duration.
// Returns a unique timer ID.
int64_t create_oneshot_timer(int32_t duration_ms);

// Cancels a timer using its ID.
void cancel_timer(int64_t timer_id);

#endif //THANATOS_TIMER_MANAGER_H
