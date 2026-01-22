#include "MediaCodecDecoder.h"

#include <iostream>

bool MediaCodecDecoder::Initialize() {
    std::cout << "MediaCodec decoder placeholder initialized.\n";
    return true;
}

void MediaCodecDecoder::SubmitPacket(const std::vector<std::uint8_t>& packet) {
    (void)packet;
    // Placeholder: feed packet to MediaCodec input buffers.
}

bool MediaCodecDecoder::DecodeNextFrame() {
    // Placeholder: dequeue output buffers and return true when a frame is ready.
    return false;
}
