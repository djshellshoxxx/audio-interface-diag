// Copyright (c) 2026 Circuit Drift Labs. All rights reserved.
// Main-thread state framing adapter. Legacy NIH VST3 stored raw JSON; NIH CLAP
// stores a little-endian u64 byte count followed by that same JSON. Never feed
// untrusted framing into NIH CLAP's allocation path without bounding it first.
#pragma once
#include <clap/stream.h>
#include "pluginterfaces/base/ibstream.h"
#include <algorithm>
#include <array>
#include <cstdint>
#include <cstring>
#include <vector>

namespace aid_state {
constexpr std::size_t limit = 16 * 1024 * 1024;

inline std::uint64_t size(const std::vector<std::uint8_t>& bytes) {
    std::uint64_t result = 0;
    for (int i = 0; i < 8; ++i) result |= std::uint64_t(bytes[i]) << (8 * i);
    return result;
}

struct Buffer {
    std::vector<std::uint8_t> bytes;
    std::size_t cursor = 0;
    clap_istream_t input{this, read};
    clap_ostream_t output{this, write};
    static std::int64_t CLAP_ABI read(const clap_istream_t* stream, void* destination, std::uint64_t count) {
        auto& self = *static_cast<Buffer*>(stream->ctx);
        const auto n = std::min<std::uint64_t>(count, self.bytes.size() - self.cursor);
        if (n) std::memcpy(destination, self.bytes.data() + self.cursor, n);
        self.cursor += n;
        return static_cast<std::int64_t>(n);
    }
    static std::int64_t CLAP_ABI write(const clap_ostream_t* stream, const void* source, std::uint64_t count) {
        auto& self = *static_cast<Buffer*>(stream->ctx);
        if (count > limit + 8 - self.bytes.size()) return -1;
        const auto* first = static_cast<const std::uint8_t*>(source);
        self.bytes.insert(self.bytes.end(), first, first + count);
        return static_cast<std::int64_t>(count);
    }
};

inline bool read(Steinberg::IBStream* source, Buffer& buffer) {
    std::array<std::uint8_t, 4096> chunk{};
    for (;;) {
        Steinberg::int32 received = 0;
        const auto result = source->read(chunk.data(), chunk.size(), &received);
        if ((result != Steinberg::kResultOk && result != Steinberg::kResultFalse)
            || received < 0 || received > static_cast<Steinberg::int32>(chunk.size())
            || static_cast<std::size_t>(received) > limit + 8 - buffer.bytes.size()) return false;
        buffer.bytes.insert(buffer.bytes.end(), chunk.begin(), chunk.begin() + received);
        if (!received || result == Steinberg::kResultFalse) break;
    }
    if (buffer.bytes.empty()) return false;
    if (buffer.bytes[0] == '{') {
        if (buffer.bytes.size() > limit) return false;
        const auto length = buffer.bytes.size();
        buffer.bytes.insert(buffer.bytes.begin(), 8, 0);
        for (int i = 0; i < 8; ++i) buffer.bytes[i] = static_cast<std::uint8_t>(length >> (8 * i));
    }
    return buffer.bytes.size() >= 9 && size(buffer.bytes) <= limit
        && size(buffer.bytes) == buffer.bytes.size() - 8 && buffer.bytes[8] == '{';
}

template<typename Plugin> bool load(Plugin& plugin, Steinberg::IBStream* source) {
    if (!plugin || !source) return false;
    Buffer buffer;
    return read(source, buffer) && plugin->load(&buffer.input);
}

template<typename Plugin> bool save(Plugin& plugin, Steinberg::IBStream* destination) {
    if (!plugin || !destination) return false;
    Buffer buffer;
    if (!plugin->save(&buffer.output) || buffer.bytes.size() < 9
        || size(buffer.bytes) != buffer.bytes.size() - 8 || buffer.bytes[8] != '{') return false;
    std::size_t cursor = 8;
    while (cursor < buffer.bytes.size()) {
        const auto count = static_cast<Steinberg::int32>(buffer.bytes.size() - cursor);
        Steinberg::int32 written = 0;
        if (destination->write(buffer.bytes.data() + cursor, count, &written) != Steinberg::kResultOk
            || written <= 0 || written > count) return false;
        cursor += written;
    }
    return true;
}
} // namespace aid_state
