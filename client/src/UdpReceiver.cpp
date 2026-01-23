#include "UdpReceiver.h"

#include "proto/udp_packet.h"

#include <array>
#include <cerrno>
#include <cstring>
#include <iostream>

#include <arpa/inet.h>
#include <netinet/in.h>
#include <sys/socket.h>
#include <unistd.h>

namespace {
constexpr std::size_t kHeaderSizeBytes = parallax::proto::kHeaderSizeBytes;
constexpr std::uint64_t kPacketLogInterval = 120;
constexpr std::uint64_t kFrameLogInterval = 30;
} // namespace

UdpReceiver::~UdpReceiver() {
    if (socket_fd_ >= 0) {
        ::close(socket_fd_);
        socket_fd_ = -1;
    }
}

bool UdpReceiver::Initialize(std::uint16_t port) {
    port_ = port;

    if (socket_fd_ >= 0) {
        ::close(socket_fd_);
        socket_fd_ = -1;
    }

    socket_fd_ = ::socket(AF_INET, SOCK_DGRAM, 0);
    if (socket_fd_ < 0) {
        std::cerr << "Failed to create UDP socket: " << std::strerror(errno) << "\n";
        return false;
    }

    sockaddr_in addr{};
    addr.sin_family = AF_INET;
    addr.sin_addr.s_addr = htonl(INADDR_ANY);
    addr.sin_port = htons(port_);

    if (::bind(socket_fd_, reinterpret_cast<sockaddr*>(&addr), sizeof(addr)) != 0) {
        std::cerr << "Failed to bind UDP socket: " << std::strerror(errno) << "\n";
        ::close(socket_fd_);
        socket_fd_ = -1;
        return false;
    }

    std::cout << "UDP receiver bound to port " << port_ << ".\n";
    return true;
}

std::vector<std::uint8_t> UdpReceiver::ReceivePacket() {
    if (socket_fd_ < 0) {
        return {};
    }

    std::array<std::uint8_t, parallax::proto::kMtuBytes> buffer{};
    while (true) {
        ssize_t received =
            ::recvfrom(socket_fd_, buffer.data(), buffer.size(), 0, nullptr, nullptr);
        if (received < 0) {
            if (errno == EINTR) {
                continue;
            }
            std::cerr << "UDP receive failed: " << std::strerror(errno) << "\n";
            return {};
        }

        if (static_cast<std::size_t>(received) < kHeaderSizeBytes) {
            continue;
        }

        parallax::proto::UdpPacketHeader header{};
        std::memcpy(&header, buffer.data(), sizeof(header));

        const std::uint32_t magic = ntohl(header.magic);
        const std::uint8_t version = header.version;
        const std::uint8_t header_length = header.header_length;
        const std::uint16_t flags = ntohs(header.flags);
        const std::uint32_t stream_id = ntohl(header.stream_id);
        const std::uint32_t frame_id = ntohl(header.frame_id);
        const std::uint16_t packet_id = ntohs(header.packet_id);
        const std::uint16_t packet_count = ntohs(header.packet_count);
        const std::uint8_t payload_type = header.payload_type;
        const std::uint8_t reserved = header.reserved;
        const std::uint16_t payload_length = ntohs(header.payload_length);

        (void)flags;
        (void)stream_id;

        if (magic != parallax::proto::kMagic || version != parallax::proto::kVersion ||
            header_length != kHeaderSizeBytes || reserved != 0 || packet_count == 0 ||
            packet_id >= packet_count) {
            continue;
        }

        if (payload_type != parallax::proto::kPayloadVideo) {
            continue;
        }

        if (payload_length > parallax::proto::kMaxPayloadSize ||
            payload_length + kHeaderSizeBytes > static_cast<std::size_t>(received)) {
            continue;
        }

        ++received_packet_count_;
        if (received_packet_count_ - last_logged_packet_count_ >= kPacketLogInterval) {
            std::cout << "UDP packets received: " << received_packet_count_ << " (frame "
                      << frame_id << ", packet " << packet_id + 1 << "/" << packet_count
                      << ").\n";
            last_logged_packet_count_ = received_packet_count_;
        }

        auto& frame = frames_[frame_id];
        if (frame.packet_count != packet_count || frame.packets.empty()) {
            frame.packet_count = packet_count;
            frame.packets.assign(packet_count, {});
            frame.received_packets = 0;
        }

        if (frame.packets[packet_id].empty()) {
            frame.packets[packet_id].assign(buffer.begin() + kHeaderSizeBytes,
                                            buffer.begin() + kHeaderSizeBytes + payload_length);
            ++frame.received_packets;
        }

        if (frame.received_packets == frame.packet_count) {
            std::size_t total_size = 0;
            for (const auto& packet : frame.packets) {
                total_size += packet.size();
            }

            std::vector<std::uint8_t> assembled;
            assembled.reserve(total_size);
            for (const auto& packet : frame.packets) {
                assembled.insert(assembled.end(), packet.begin(), packet.end());
            }

            frames_.erase(frame_id);
            ++assembled_frame_count_;
            if (assembled_frame_count_ - last_logged_frame_count_ >= kFrameLogInterval) {
                std::cout << "Assembled frames: " << assembled_frame_count_ << " ("
                          << received_packet_count_ << " packets).\n";
                last_logged_frame_count_ = assembled_frame_count_;
            }
            return assembled;
        }
    }
}
