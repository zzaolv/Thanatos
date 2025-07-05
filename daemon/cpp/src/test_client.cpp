// 文件路径: /Thanatos/daemon/cpp/src/test_client.cpp
// This is an optional utility for testing the daemon on the build server
// without needing a full Android App.

#include <iostream>
#include <memory>
#include <string>
#include <grpcpp/grpcpp.h>

#include "thanatos.grpc.pb.h"

using grpc::Channel;
using grpc::ClientContext;
using grpc::Status;
using grpc::ClientReader;

using thanatos::ipc::DashboardService;
using thanatos::ipc::RuntimeStats;
using google::protobuf::Empty;

// Use /tmp for testing on a standard Linux host.
const char* SOCKET_PATH = "/tmp/thanatosd.sock";

class DashboardClient {
public:
    DashboardClient(std::shared_ptr<Channel> channel)
        : stub_(DashboardService::NewStub(channel)) {}

    void GetRuntimeStatsStream() {
        ClientContext context;
        Empty request;
        RuntimeStats stats;

        std::unique_ptr<ClientReader<RuntimeStats>> reader(
            stub_->StreamRuntimeStats(&context, request));

        std::cout << "Client: Calling StreamRuntimeStats..." << std::endl;
        
        while (reader->Read(&stats)) {
            std::cout << "--------------------------------" << std::endl;
            std::cout << "Received Stats at " << std::time(nullptr) << ":" << std::endl;
            std::cout << "  Total Memory: " << stats.total_mem_kb() / 1024 << " MB" << std::endl;
            std::cout << "  Avail Memory: " << stats.avail_mem_kb() / 1024 << " MB" << std::endl;
            std::cout << "  Frozen Apps:  " << stats.frozen_app_count() << std::endl;
            std::cout << "  CPU Usage:    " << stats.cpu_usage_percent() << "%" << std::endl;
            std::cout << "  Net Speed:    " << stats.network_speed_bps() / 1000 << " Kbps" << std::endl;
            std::cout << "--------------------------------" << std::endl;
        }

        Status status = reader->Finish();
        if (status.ok()) {
            std::cout << "Client: Stream finished successfully." << std::endl;
        } else {
            std::cout << "Client: RPC failed. " << status.error_code() << ": " << status.error_message()
                      << std::endl;
        }
    }

private:
    std::unique_ptr<DashboardService::Stub> stub_;
};

int main(int argc, char** argv) {
    std::string server_address = "unix:" + std::string(SOCKET_PATH);
    auto channel = grpc::CreateChannel(server_address, grpc::InsecureChannelCredentials());
    DashboardClient client(channel);

    client.GetRuntimeStatsStream();

    return 0;
}
