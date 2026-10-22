cmake_minimum_required(VERSION 3.24)
project(aid_vst3_validation LANGUAGES C CXX)
set(SMTG_ENABLE_VST3_HOSTING_EXAMPLES OFF CACHE BOOL "No interactive hosts")
set(SMTG_ENABLE_VST3_PLUGIN_EXAMPLES OFF CACHE BOOL "No example plugins")
set(SMTG_ENABLE_VSTGUI_SUPPORT OFF CACHE BOOL "No GUI SDK")
add_subdirectory("${VST3_SDK_ROOT}" sdk EXCLUDE_FROM_ALL)
add_executable(aid_compatibility compatibility.cpp
    "${VST3_SDK_ROOT}/public.sdk/source/vst/hosting/plugprovider.cpp"
    "${VST3_SDK_ROOT}/public.sdk/source/common/memorystream.cpp")
if(APPLE)
    enable_language(OBJCXX)
    target_sources(aid_compatibility PRIVATE "${VST3_SDK_ROOT}/public.sdk/source/vst/hosting/module_mac.mm")
    # The SDK's macOS module loader owns Cocoa objects using ARC.
    target_compile_options(aid_compatibility PRIVATE "$<$<COMPILE_LANGUAGE:OBJCXX>:-fobjc-arc>")
    target_link_libraries(aid_compatibility PRIVATE "-framework Cocoa")
elseif(WIN32)
    target_sources(aid_compatibility PRIVATE "${VST3_SDK_ROOT}/public.sdk/source/vst/hosting/module_win32.cpp")
else()
    target_sources(aid_compatibility PRIVATE "${VST3_SDK_ROOT}/public.sdk/source/vst/hosting/module_linux.cpp")
endif()
target_compile_features(aid_compatibility PRIVATE cxx_std_17)
target_link_libraries(aid_compatibility PRIVATE sdk_hosting)
# A root project with explicit references is required by Visual Studio generators;
# directly naming the nested EXCLUDE_FROM_ALL validator project is not portable.
add_custom_target(aid_validation_tools DEPENDS aid_compatibility validator)
