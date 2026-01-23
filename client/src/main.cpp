#include "OpenXRApp.h"
#include "UdpReceiver.h"

#include <cstdint>
#include <fstream>
#include <iostream>
#include <string>

namespace {
constexpr std::uint16_t kDefaultUdpPort = 7777;
} // namespace

int main(int argc, char* argv[]) {
    bool dump_h264 = false;
    std::string dump_path;

    for (int i = 1; i < argc; ++i) {
        std::string arg(argv[i]);
        if (arg == "--dump-h264") {
            dump_h264 = true;
            if (i + 1 < argc && argv[i + 1][0] != '-') {
                dump_path = argv[++i];
            }
        }
    }

    if (dump_h264) {
        UdpReceiver receiver;
        if (!receiver.Initialize(kDefaultUdpPort)) {
            std::cerr << "Failed to initialize UDP receiver." << std::endl;
            return 1;
        }

        std::ofstream file_stream;
        std::ostream* output = &std::cout;
        if (!dump_path.empty()) {
            file_stream.open(dump_path, std::ios::binary);
            if (!file_stream) {
                std::cerr << "Failed to open dump path: " << dump_path << std::endl;
                return 1;
            }
            output = &file_stream;
        } else {
            std::cout.sync_with_stdio(false);
        }

        while (true) {
            auto packet = receiver.ReceivePacket();
            if (!packet.empty()) {
                output->write(reinterpret_cast<const char*>(packet.data()),
                              static_cast<std::streamsize>(packet.size()));
                output->flush();
            }
        }
    }

    OpenXRApp app;
    if (!app.Initialize()) {
        std::cerr << "Failed to initialize Parallax OpenXR app." << std::endl;
        return 1;
    }

    app.Run();
    app.Shutdown();
    return 0;
}
