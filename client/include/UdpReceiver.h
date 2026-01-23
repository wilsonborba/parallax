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
    std::uint64_t received_packet_count_ = 0;
    std::uint64_t assembled_frame_count_ = 0;
    std::uint64_t last_logged_packet_count_ = 0;
    std::uint64_t last_logged_frame_count_ = 0;
};
