// Copyright (c) 2026 Circuit Drift Labs. All rights reserved.
#include "public.sdk/source/vst/hosting/module.h"
#include "public.sdk/source/vst/hosting/plugprovider.h"
#include "public.sdk/source/vst/hosting/hostclasses.h"
#include "public.sdk/source/common/memorystream.h"
#include <cmath>
#include <fstream>
#include <iostream>
#include <stdexcept>
#include <string>
#include <vector>
using namespace Steinberg;
using namespace Steinberg::Vst;

int main(int argc, char** argv) {
    try {
        if (argc != 4) throw std::runtime_error("usage: aid_compatibility <bundle> <legacy-state> <save|load>");
        std::string error;
        auto module = VST3::Hosting::Module::create(argv[1], error);
        if (!module) throw std::runtime_error(error);
        auto classes = module->getFactory().classInfos();
        if (classes.empty()) throw std::runtime_error("No plugin classes");
        if (classes[0].ID().toString() != "43444C417564696F4966616365446961") throw std::runtime_error("Component identity changed");
        HostApplication host;
        PluginContextFactory::instance().setPluginContext(&host);
        PlugProvider provider(module->getFactory(), classes[0], true);
        if (!provider.initialize()) throw std::runtime_error("Plugin initialization failed");
        auto component = provider.getComponentPtr();
        auto controller = provider.getControllerPtr();
        const uint32 expected[] = {3357091,1804913353,1804489223,109760981,1804390113,1213087024,918121231,1237388699};
        if (controller->getParameterCount() != 8) throw std::runtime_error("Parameter count changed");
        for (int32 i = 0; i < 8; ++i) {
            ParameterInfo info;
            if (controller->getParameterInfo(i, info) != kResultOk || info.id != expected[i]) throw std::runtime_error("Automation identity changed");
        }
        if (std::string(argv[3]) == "save") {
            controller->setParamNormalized(expected[0], 0.5);
            controller->setParamNormalized(expected[2], 0.75);
            controller->setParamNormalized(expected[3], 0.2);
            MemoryStream state;
            if (component->getState(&state) != kResultOk) throw std::runtime_error("Legacy state save failed");
            std::ofstream file(argv[2], std::ios::binary);
            file.write(state.getData(), state.getSize());
            if (!file) throw std::runtime_error("Fixture write failed");
        } else {
            std::ifstream file(argv[2], std::ios::binary);
            std::vector<char> bytes((std::istreambuf_iterator<char>(file)), {});
            if (bytes.empty()) throw std::runtime_error("Missing legacy fixture");
            MemoryStream state(bytes.data(), bytes.size());
            if (component->setState(&state) != kResultOk) throw std::runtime_error("Legacy state restore failed");
            const auto check = [&] {
                if (std::abs(controller->getParamNormalized(expected[0]) - 0.5) > 1e-5 ||
                    std::abs(controller->getParamNormalized(expected[2]) - 0.75) > 1e-5 ||
                    std::abs(controller->getParamNormalized(expected[3]) - 0.2) > 1e-5) throw std::runtime_error("Restored parameter values differ from legacy state");
            };
            check();
            if (component->setState(nullptr) == kResultOk || component->getState(nullptr) == kResultOk) throw std::runtime_error("Null host stream was accepted");
            check();
            std::vector<char> framed(8, 0);
            const auto length = bytes.size();
            for (int i = 0; i < 8; ++i) framed[i] = static_cast<char>(length >> (8 * i));
            framed.insert(framed.end(), bytes.begin(), bytes.end());
            MemoryStream transitional(framed.data(), framed.size());
            if (component->setState(&transitional) != kResultOk) throw std::runtime_error("Bounded CLAP-framed state restore failed");
            check();
            for (const auto& invalid : std::vector<std::vector<char>>{{}, {'{'}, {'n','o','t','j','s','o','n'}, std::vector<char>(8, static_cast<char>(0xff)), {1,0,0,0,0,0,0,0}}) {
                MemoryStream corrupt(const_cast<char*>(invalid.data()), invalid.size());
                if (component->setState(&corrupt) == kResultOk) throw std::runtime_error("Invalid or truncated state was accepted");
                check();
            }
            std::vector<char> oversized(16 * 1024 * 1024 + 9, ' ');
            oversized[0] = '{';
            MemoryStream excessive(oversized.data(), oversized.size());
            if (component->setState(&excessive) == kResultOk) throw std::runtime_error("Oversized state was accepted");
            check();
            MemoryStream again;
            if (component->getState(&again) != kResultOk || again.getSize() == 0 || again.getData()[0] != '{') throw std::runtime_error("State resave failed");
            again.seek(0, IBStream::kIBSeekSet, nullptr);
            if (component->setState(&again) != kResultOk) throw std::runtime_error("State roundtrip failed");
            check();
        }
        std::cout << "Component ID, eight automation IDs and legacy state values verified\n";
        return 0;
    } catch (const std::exception& error) {
        std::cerr << error.what() << '\n';
        return 1;
    }
}
