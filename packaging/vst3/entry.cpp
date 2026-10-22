// Copyright (c) 2026 Circuit Drift Labs. All rights reserved.
// Self-contained CLAP entry proxy for the MIT VST3 wrapper. This adapter performs
// no audio work; plugin factories and processing are owned by the NIH CLAP.
#include <clap/entry.h>
#include "detail/os/osutil.h"
#include <filesystem>
#include <string>
#include <mutex>
#if defined(_WIN32)
#include <windows.h>
#else
#include <dlfcn.h>
#endif

namespace {
#if defined(_WIN32)
HMODULE library = nullptr;
#else
void* library = nullptr;
#endif
const clap_plugin_entry_t* implementation = nullptr;
std::mutex lifecycle;
std::size_t references = 0;

void unload() {
    implementation = nullptr;
    if (library) {
#if defined(_WIN32)
        FreeLibrary(library);
#else
        dlclose(library);
#endif
        library = nullptr;
    }
}

bool CLAP_ABI initialize(const char*) {
    const std::lock_guard<std::mutex> lock(lifecycle);
    if (references) { ++references; return true; }
    // Every platform's wrapper binary is Contents/<architecture>/<binary>.
    // The private CLAP lives inside this exact bundle, never a search path.
    const auto resources = os::getPluginPath().parent_path().parent_path() / "Resources";
    auto binary = resources / "Audio Interface Diag.clap";
#if defined(__APPLE__)
    binary /= "Contents/MacOS/Audio Interface Diag";
#endif
#if defined(_WIN32)
    library = LoadLibraryW(binary.c_str());
    if (library) implementation = reinterpret_cast<const clap_plugin_entry_t*>(GetProcAddress(library, "clap_entry"));
#else
    library = dlopen(binary.c_str(), RTLD_NOW | RTLD_LOCAL);
    if (library) implementation = static_cast<const clap_plugin_entry_t*>(dlsym(library, "clap_entry"));
#endif
    if (!implementation || !clap_version_is_compatible(implementation->clap_version)
        || !implementation->init || !implementation->deinit || !implementation->get_factory) {
        unload();
        return false;
    }
    if (!implementation->init(binary.u8string().c_str())) {
        unload();
        return false;
    }
    references = 1;
    return true;
}

void CLAP_ABI deinitialize() {
    const std::lock_guard<std::mutex> lock(lifecycle);
    if (!references || --references) return;
    if (implementation) implementation->deinit();
    unload();
}

const void* CLAP_ABI factory(const char* id) {
    const std::lock_guard<std::mutex> lock(lifecycle);
    return implementation ? implementation->get_factory(id) : nullptr;
}
} // namespace

extern "C" CLAP_EXPORT const clap_plugin_entry_t clap_entry = {
    CLAP_VERSION_INIT, initialize, deinitialize, factory
};
