#pragma once

#include <cstdint>

#if defined(__has_include)
#if __has_include(<openxr/openxr.h>)
#include <openxr/openxr.h>
#include <openxr/openxr_platform.h>
#define PARALLAX_HAS_OPENXR_HEADERS 1
#else
#define PARALLAX_HAS_OPENXR_HEADERS 0
#endif
#else
#define PARALLAX_HAS_OPENXR_HEADERS 0
#endif

#if !PARALLAX_HAS_OPENXR_HEADERS
using XrInstance = std::uint64_t;
using XrSystemId = std::uint64_t;
using XrSession = std::uint64_t;
using XrResult = std::int32_t;
using XrBool32 = std::uint32_t;

constexpr XrInstance XR_NULL_HANDLE = 0;
constexpr XrSystemId XR_NULL_SYSTEM_ID = 0;
constexpr XrSession XR_NULL_SESSION = 0;
constexpr XrResult XR_SUCCESS = 0;
constexpr XrBool32 XR_FALSE = 0;
constexpr XrBool32 XR_TRUE = 1;

struct XrInstanceCreateInfo {
    std::uint32_t type;
    const void* next;
    std::uint32_t createFlags;
    char applicationName[128];
    std::uint32_t applicationVersion;
    char engineName[128];
    std::uint32_t engineVersion;
    std::uint32_t apiVersion;
};

struct XrSystemGetInfo {
    std::uint32_t type;
    const void* next;
    std::uint32_t formFactor;
};

constexpr std::uint32_t XR_TYPE_INSTANCE_CREATE_INFO = 1;
constexpr std::uint32_t XR_TYPE_SYSTEM_GET_INFO = 2;
constexpr std::uint32_t XR_FORM_FACTOR_HEAD_MOUNTED_DISPLAY = 1;
constexpr std::uint32_t XR_CURRENT_API_VERSION = 1;

inline XrResult xrCreateInstance(const XrInstanceCreateInfo*, XrInstance*) {
    return XR_SUCCESS;
}

inline XrResult xrGetSystem(XrInstance, const XrSystemGetInfo*, XrSystemId*) {
    return XR_SUCCESS;
}

inline XrResult xrDestroyInstance(XrInstance) {
    return XR_SUCCESS;
}
#endif
