#include "OpenXRApp.h"

#include <iostream>

int main() {
    OpenXRApp app;
    if (!app.Initialize()) {
        std::cerr << "Failed to initialize Parallax OpenXR app." << std::endl;
        return 1;
    }

    app.Run();
    app.Shutdown();
    return 0;
}
