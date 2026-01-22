#pragma once

#include <cstdint>
#include <string>
#include <vector>

class UdpReceiver {
public:
    bool Initialize(std::uint16_t port);
    std::vector<std::uint8_t> ReceivePacket();

private:
    std::uint16_t port_ = 0;
};
