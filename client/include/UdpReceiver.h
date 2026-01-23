#pragma once

#include <cstdint>
#include <string>
#include <unordered_map>
#include <vector>

class UdpReceiver {
public:
    bool Initialize(std::uint16_t port);
    std::vector<std::uint8_t> ReceivePacket();
    ~UdpReceiver();

private:
    struct FrameAssembly {
        std::uint16_t packet_count = 0;
        std::vector<std::vector<std::uint8_t>> packets;
        std::size_t received_packets = 0;
    };

    std::uint16_t port_ = 0;
    int socket_fd_ = -1;
    std::unordered_map<std::uint32_t, FrameAssembly> frames_;
};
