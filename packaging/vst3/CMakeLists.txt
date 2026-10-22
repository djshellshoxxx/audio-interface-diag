cmake_minimum_required(VERSION 3.24)
project(aid_vst3 VERSION 0.0.4 LANGUAGES C CXX)
set(CMAKE_CXX_STANDARD 17)
if(APPLE)
    enable_language(OBJC OBJCXX)
endif()
set(CMAKE_POSITION_INDEPENDENT_CODE ON)
set(CMAKE_CXX_VISIBILITY_PRESET hidden)
set(CMAKE_VISIBILITY_INLINES_HIDDEN ON)
set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreaded$<$<CONFIG:Debug>:Debug>")
set(CLAP_WRAPPER_DOWNLOAD_DEPENDENCIES OFF CACHE BOOL "Use only pinned local SDKs")
set(CLAP_WRAPPER_COPY_AFTER_BUILD OFF CACHE BOOL "Never install into user directories")
set(CLAP_WRAPPER_OUTPUT_NAME "Audio Interface Diag")
set(CLAP_WRAPPER_BUNDLE_VERSION "0.0.4")
set(CLAP_WRAPPER_BUILD_AAX OFF CACHE BOOL "No AAX SDK")
set(CLAP_WRAPPER_BUILD_STANDALONE OFF CACHE BOOL "NIH supplies standalone")
add_subdirectory("${AID_WRAPPER_ROOT}" wrapper EXCLUDE_FROM_ALL)
# Directly resolve our entry proxy; the wrapper must not discover unrelated CLAPs.
target_compile_definitions(clap-wrapper-shared-detail PRIVATE STATICALLY_LINKED_CLAP_ENTRY=1)
add_library(aid_vst3 MODULE entry.cpp)
target_include_directories(aid_vst3 PRIVATE "${AID_WRAPPER_ROOT}/src")
# Preserve the NIH component ID's canonical bytes. Steinberg's FUID performs the
# platform byte-order conversion, matching NIH's PLATFORM_VST3_CLASS_ID.
target_add_vst3_wrapper(TARGET aid_vst3 OUTPUT_NAME "Audio Interface Diag"
    SINGLE_PLUGIN_TUID "43444C417564696F4966616365446961"
    BUNDLE_IDENTIFIER "com.circuitdriftlabs.audio-interface-diag.vst3"
    BUNDLE_VERSION "0.0.4" WINDOWS_FOLDER_VST3 TRUE)

# Replace only the two state call sites in a generated copy of the pinned MIT
# translation unit. Upstream checkouts stay untouched; unexpected upstream source
# changes fail configuration rather than silently dropping compatibility.
set(upstream_state_source "${AID_WRAPPER_ROOT}/src/wrapasvst3.cpp")
file(READ "${upstream_state_source}" adapted_state_source)
foreach(call IN ITEMS "_plugin->load(CLAPVST3StreamAdapter(state))" "_plugin->save(CLAPVST3StreamAdapter(state))")
    string(FIND "${adapted_state_source}" "${call}" position)
    if(position EQUAL -1)
        message(FATAL_ERROR "Pinned wrapper state call changed: ${call}")
    endif()
endforeach()
string(REPLACE "_plugin->load(CLAPVST3StreamAdapter(state))" "aid_state::load(_plugin, state)" adapted_state_source "${adapted_state_source}")
string(REPLACE "_plugin->save(CLAPVST3StreamAdapter(state))" "aid_state::save(_plugin, state)" adapted_state_source "${adapted_state_source}")
file(WRITE "${CMAKE_CURRENT_BINARY_DIR}/aid_vst3_state.cpp" "#include \"aid_state.h\"\n${adapted_state_source}")
get_target_property(wrapper_sources aid_vst3-clap-wrapper-vst3-lib SOURCES)
list(REMOVE_ITEM wrapper_sources "${upstream_state_source}")
set_property(TARGET aid_vst3-clap-wrapper-vst3-lib PROPERTY SOURCES "${wrapper_sources}")
target_sources(aid_vst3-clap-wrapper-vst3-lib PRIVATE "${CMAKE_CURRENT_BINARY_DIR}/aid_vst3_state.cpp")
target_include_directories(aid_vst3-clap-wrapper-vst3-lib PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}" "${AID_WRAPPER_ROOT}/src")
