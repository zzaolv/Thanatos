// 文件路径: /Thanatos/daemon/cpp/src/grpc_server.cpp

#include "grpc_server.h"
#include <iostream>
#include <memory>
#include <thread>
#include <chrono>

#include <grpcpp/grpcpp.h>
#include <grpcpp/health_check_service_interface.h>

// Include generated headers
#include "thanatos.grpc.pb.h"
#include "thanatos_logic_bridge/src/ffi.rs.h"

// Forward declaration of the global Rust controller object
extern std::unique_ptr<LogicController> controller;

using grpc::Server;
using grpc::ServerBuilder;
using grpc::ServerContext;
using grpc::Status;
using grpc::ServerReader;
using grpc::ServerWriter;
using google::protobuf::Empty;

// --- Service Implementations ---

class AppControlServiceImpl final : public thanatos::ipc::AppControlService::Service {
public:
    Status SetAppConfig(ServerContext* context, const thanatos::ipc::AppConfig* request, Empty* response) override {
        std::cout << "[C++ gRPC] SetAppConfig for " << request->package_name() << std::endl;
        
        ffi::CxxAppConfig cxx_config;
        cxx_config.package_name = request->package_name();
        cxx_config.policy = static_cast<int32_t>(request->policy());
        cxx_config.freeze_mode = static_cast<int32_t>(request->freeze_mode());
        cxx_config.oom_priority = static_cast<int32_t>(request->oom_priority());
        cxx_config.network_policy = static_cast<int32_t>(request->network_policy());
        cxx_config.allow_wakeup_for_push = request->allow_wakeup_for_push();
        cxx_config.allow_autostart = request->allow_autostart();
        
        rust_set_app_config(*controller, cxx_config);
        
        return Status::OK;
    }

    Status SetLaunchRule(ServerContext* context, const thanatos::ipc::LaunchRule* request, Empty* response) override {
        ffi::CxxLaunchRule cxx_rule;
        cxx_rule.source_package = request->source_package();
        cxx_rule.target_package = request->target_package();
        cxx_rule.allowed = request->allowed();
        
        rust_set_launch_rule(*controller, cxx_rule);
        
        return Status::OK;
    }
};

class SystemServiceImpl final : public thanatos::ipc::SystemService::Service {
public:
    Status PushFrameworkEvents(ServerContext* context, ServerReader<thanatos::ipc::FrameworkEvent>* reader) override {
        thanatos::ipc::FrameworkEvent event;
        while (reader->Read(&event)) {
            if (context->IsCancelled()) break;

            ffi::CxxFrameworkEvent cxx_event;
            cxx_event.event_type = static_cast<int32_t>(event.event_type());
            cxx_event.source_package = event.source_package();
            cxx_event.target_package = event.target_package();
            cxx_event.timestamp = event.timestamp();
            
            rust_handle_framework_event(*controller, cxx_event);
        }
        return Status::OK;
    }

    Status RequestTemporaryUnfreeze(ServerContext* context, const thanatos::ipc::TempUnfreezeRequest* request, Empty* response) override {
        rust_request_temporary_unfreeze(
            *controller,
            request->package_name(),
            static_cast<int32_t>(request->reason()),
            request->duration_ms()
        );
        return Status::OK;
    }

    Status ShouldAllowLaunch(ServerContext* context, const thanatos::ipc::LaunchRule* request, thanatos::ipc::LaunchRule* response) override {
        ffi::CxxLaunchRule cxx_request_rule;
        cxx_request_rule.source_package = request->source_package();
        cxx_request_rule.target_package = request->target_package();
        
        auto cxx_response_rule = rust_should_allow_launch(*controller, cxx_request_rule);

        response->set_source_package(cxx_response_rule.source_package);
        response->set_target_package(cxx_response_rule.target_package);
        response->set_allowed(cxx_response_rule.allowed);

        return Status::OK;
    }
};

class DashboardServiceImpl final : public thanatos::ipc::DashboardService::Service {
public:
    Status StreamRuntimeStats(ServerContext* context, const Empty* request, ServerWriter<thanatos::ipc::RuntimeStats>* writer) override {
        std::cout << "[C++ gRPC] Client subscribed to StreamRuntimeStats." << std::endl;
        
        while (!context->IsCancelled()) {
            auto rust_stats = rust_get_runtime_stats(*controller);
            
            thanatos::ipc::RuntimeStats stats;
            stats.set_total_mem_kb(rust_stats.total_mem_kb);
            stats.set_avail_mem_kb(rust_stats.avail_mem_kb);
            stats.set_frozen_app_count(rust_stats.frozen_app_count);
            stats.set_cpu_usage_percent(rust_stats.cpu_usage_percent);
            stats.set_network_speed_bps(rust_stats.network_speed_bps);

            if (!writer->Write(stats)) {
                break; // Client disconnected
            }

            // Sleep for 1 second
            std::this_thread::sleep_for(std::chrono::seconds(1));
        }

        std::cout << "[C++ gRPC] Client disconnected from StreamRuntimeStats." << std::endl;
        return Status::OK;
    }
    
    Status GetRecentEvents(ServerContext* context, const Empty* request, ServerWriter<thanatos::ipc::EventLog>* writer) override {
        auto events = rust_get_recent_events(*controller, 50); // Get last 50 events
        for (const auto& event : events) {
            if (context->IsCancelled()) break;
            
            thanatos::ipc::EventLog log;
            log.set_timestamp(event.timestamp);
            log.set_package_name(event.package_name);
            log.set_event_description(event.description);
            
            if (!writer->Write(log)) {
                break; // Client disconnected
            }
        }
        return Status::OK;
    }
};

// --- Server Main Function ---

void RunServer(const std::string& socket_path) {
    std::string server_address = "unix:" + socket_path;
    
    AppControlServiceImpl app_service;
    SystemServiceImpl system_service;
    DashboardServiceImpl dashboard_service;

    grpc::EnableDefaultHealthCheckService(true);
    ServerBuilder builder;

    builder.AddListeningPort(server_address, grpc::InsecureServerCredentials());
    
    builder.RegisterService(&app_service);
    builder.RegisterService(&system_service);
    builder.RegisterService(&dashboard_service;

    std::unique_ptr<Server> server(builder.BuildAndStart());
    std::cout << "[C++ gRPC] Server listening on " << server_address << std::endl;

    // This will block the calling thread (which is intended for the gRPC thread)
    server->Wait();
}
