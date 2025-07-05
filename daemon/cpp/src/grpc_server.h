// 文件路径: /Thanatos/daemon/cpp/src/grpc_server.h
#ifndef THANATOS_GRPC_SERVER_H
#define THANATOS_GRPC_SERVER_H

#include <string>

// Starts the gRPC server. This function will block until the server is shut down.
void RunServer(const std::string& socket_path);

#endif //THANATOS_GRPC_SERVER_H
