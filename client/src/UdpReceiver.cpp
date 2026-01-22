#include "UdpReceiver.h"

#include <iostream>

bool UdpReceiver::Initialize(std::uint16_t port) {
    port_ = port;
    std::cout << "UDP receiver placeholder bound to port " << port_ << ".\n";
    return true;
}

std::vector<std::uint8_t> UdpReceiver::ReceivePacket() {
    // Placeholder: replace with real UDP socket receive.
    return {};
}
