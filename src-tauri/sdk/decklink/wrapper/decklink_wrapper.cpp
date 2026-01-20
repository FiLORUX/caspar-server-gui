/**
 * DeckLink SDK C Wrapper Implementation
 *
 * Windows COM implementation for enumerating DeckLink devices.
 * Requires DeckLink Desktop Video drivers to be installed.
 *
 * Copyright (c) 2026 Thast. MIT License.
 */

#ifdef _WIN32

#include "decklink_wrapper.h"
#include <comdef.h>
#include <string>

// Include the generated DeckLink API header (from IDL via MIDL)
#include "DeckLinkAPI_h.h"

static bool g_initialised = false;

/**
 * Convert BSTR to C string (UTF-8)
 */
static void bstr_to_cstr(BSTR bstr, char* out, int max_length) {
    if (bstr == nullptr || out == nullptr || max_length <= 0) {
        if (out && max_length > 0) out[0] = '\0';
        return;
    }

    int wlen = SysStringLen(bstr);
    if (wlen == 0) {
        out[0] = '\0';
        return;
    }

    // Convert wide string to UTF-8
    int utf8len = WideCharToMultiByte(CP_UTF8, 0, bstr, wlen, nullptr, 0, nullptr, nullptr);
    if (utf8len <= 0 || utf8len >= max_length) {
        // Fallback to truncated conversion
        utf8len = max_length - 1;
    }

    WideCharToMultiByte(CP_UTF8, 0, bstr, wlen, out, utf8len, nullptr, nullptr);
    out[utf8len] = '\0';
}

extern "C" {

DeckLinkError decklink_init(void) {
    if (g_initialised) {
        return DECKLINK_OK;
    }

    HRESULT hr = CoInitializeEx(nullptr, COINIT_MULTITHREADED);
    if (FAILED(hr) && hr != RPC_E_CHANGED_MODE) {
        return DECKLINK_ERROR_COM_FAILED;
    }

    g_initialised = true;
    return DECKLINK_OK;
}

void decklink_cleanup(void) {
    if (g_initialised) {
        CoUninitialize();
        g_initialised = false;
    }
}

DeckLinkError decklink_get_device_count(int32_t* count) {
    if (!g_initialised) {
        return DECKLINK_ERROR_NOT_INITIALISED;
    }

    if (count == nullptr) {
        return DECKLINK_ERROR_INVALID_INDEX;
    }

    *count = 0;

    IDeckLinkIterator* iterator = nullptr;
    HRESULT hr = CoCreateInstance(
        CLSID_CDeckLinkIterator,
        nullptr,
        CLSCTX_ALL,
        IID_IDeckLinkIterator,
        reinterpret_cast<void**>(&iterator)
    );

    if (FAILED(hr) || iterator == nullptr) {
        return DECKLINK_ERROR_NO_DRIVER;
    }

    IDeckLink* deckLink = nullptr;
    int32_t deviceCount = 0;

    while (iterator->Next(&deckLink) == S_OK) {
        deviceCount++;
        deckLink->Release();
    }

    iterator->Release();
    *count = deviceCount;

    return DECKLINK_OK;
}

DeckLinkError decklink_get_device_info(int32_t index, DeckLinkDeviceInfo* info) {
    if (!g_initialised) {
        return DECKLINK_ERROR_NOT_INITIALISED;
    }

    if (info == nullptr || index < 0) {
        return DECKLINK_ERROR_INVALID_INDEX;
    }

    // Zero out the structure
    memset(info, 0, sizeof(DeckLinkDeviceInfo));
    info->index = index;
    info->persistent_id = -1;
    info->device_group_id = -1;

    IDeckLinkIterator* iterator = nullptr;
    HRESULT hr = CoCreateInstance(
        CLSID_CDeckLinkIterator,
        nullptr,
        CLSCTX_ALL,
        IID_IDeckLinkIterator,
        reinterpret_cast<void**>(&iterator)
    );

    if (FAILED(hr) || iterator == nullptr) {
        return DECKLINK_ERROR_NO_DRIVER;
    }

    IDeckLink* deckLink = nullptr;
    int32_t currentIndex = 0;
    DeckLinkError result = DECKLINK_ERROR_INVALID_INDEX;

    while (iterator->Next(&deckLink) == S_OK) {
        if (currentIndex == index) {
            // Found the device, get info
            BSTR displayName = nullptr;
            BSTR modelName = nullptr;

            // Get display name
            if (deckLink->GetDisplayName(&displayName) == S_OK && displayName) {
                bstr_to_cstr(displayName, info->display_name, DECKLINK_MAX_STRING_LENGTH);
                SysFreeString(displayName);
            }

            // Get model name
            if (deckLink->GetModelName(&modelName) == S_OK && modelName) {
                bstr_to_cstr(modelName, info->model_name, DECKLINK_MAX_STRING_LENGTH);
                SysFreeString(modelName);
            }

            // Query profile attributes interface
            IDeckLinkProfileAttributes* attributes = nullptr;
            if (deckLink->QueryInterface(IID_IDeckLinkProfileAttributes, reinterpret_cast<void**>(&attributes)) == S_OK && attributes) {

                // Persistent ID
                int64_t persistentId = 0;
                if (attributes->GetInt(BMDDeckLinkPersistentID, &persistentId) == S_OK) {
                    info->persistent_id = persistentId;
                }

                // Device group ID
                int64_t groupId = 0;
                if (attributes->GetInt(BMDDeckLinkDeviceGroupID, &groupId) == S_OK) {
                    info->device_group_id = groupId;
                }

                // Sub-device index
                int64_t subDeviceIndex = 0;
                if (attributes->GetInt(BMDDeckLinkSubDeviceIndex, &subDeviceIndex) == S_OK) {
                    info->sub_device_index = static_cast<int32_t>(subDeviceIndex);
                }

                // Number of sub-devices
                int64_t numSubDevices = 0;
                if (attributes->GetInt(BMDDeckLinkNumberOfSubDevices, &numSubDevices) == S_OK) {
                    info->num_sub_devices = static_cast<int32_t>(numSubDevices);
                }

                // Video input connections
                int64_t videoInputConnections = 0;
                if (attributes->GetInt(BMDDeckLinkVideoInputConnections, &videoInputConnections) == S_OK) {
                    info->video_input_connections = static_cast<uint32_t>(videoInputConnections);
                }

                // Video output connections
                int64_t videoOutputConnections = 0;
                if (attributes->GetInt(BMDDeckLinkVideoOutputConnections, &videoOutputConnections) == S_OK) {
                    info->video_output_connections = static_cast<uint32_t>(videoOutputConnections);
                }

                // Audio input connections
                int64_t audioInputConnections = 0;
                if (attributes->GetInt(BMDDeckLinkAudioInputConnections, &audioInputConnections) == S_OK) {
                    info->audio_input_connections = static_cast<uint32_t>(audioInputConnections);
                }

                // Audio output connections
                int64_t audioOutputConnections = 0;
                if (attributes->GetInt(BMDDeckLinkAudioOutputConnections, &audioOutputConnections) == S_OK) {
                    info->audio_output_connections = static_cast<uint32_t>(audioOutputConnections);
                }

                // IO support
                int64_t ioSupport = 0;
                if (attributes->GetInt(BMDDeckLinkVideoIOSupport, &ioSupport) == S_OK) {
                    info->io_support = static_cast<uint32_t>(ioSupport);
                }

                // Keying support
                BOOL supportsInternalKeying = FALSE;
                if (attributes->GetFlag(BMDDeckLinkSupportsInternalKeying, &supportsInternalKeying) == S_OK) {
                    info->supports_internal_keying = supportsInternalKeying != FALSE;
                }

                BOOL supportsExternalKeying = FALSE;
                if (attributes->GetFlag(BMDDeckLinkSupportsExternalKeying, &supportsExternalKeying) == S_OK) {
                    info->supports_external_keying = supportsExternalKeying != FALSE;
                }

                // SDI link support
                BOOL supportsDualLink = FALSE;
                if (attributes->GetFlag(BMDDeckLinkSupportsDualLinkSDI, &supportsDualLink) == S_OK) {
                    info->supports_dual_link_sdi = supportsDualLink != FALSE;
                }

                BOOL supportsQuadLink = FALSE;
                if (attributes->GetFlag(BMDDeckLinkSupportsQuadLinkSDI, &supportsQuadLink) == S_OK) {
                    info->supports_quad_link_sdi = supportsQuadLink != FALSE;
                }

                // Idle output support
                BOOL supportsIdleOutput = FALSE;
                if (attributes->GetFlag(BMDDeckLinkSupportsIdleOutput, &supportsIdleOutput) == S_OK) {
                    info->supports_idle_output = supportsIdleOutput != FALSE;
                }

                // Max audio channels
                int64_t maxAudioChannels = 0;
                if (attributes->GetInt(BMDDeckLinkMaximumAudioChannels, &maxAudioChannels) == S_OK) {
                    info->max_audio_channels = static_cast<int32_t>(maxAudioChannels);
                }

                attributes->Release();
            }

            // Query configuration interface for device label
            IDeckLinkConfiguration* config = nullptr;
            if (deckLink->QueryInterface(IID_IDeckLinkConfiguration, reinterpret_cast<void**>(&config)) == S_OK && config) {
                BSTR deviceLabel = nullptr;
                if (config->GetString(bmdDeckLinkConfigDeviceInformationLabel, &deviceLabel) == S_OK && deviceLabel) {
                    bstr_to_cstr(deviceLabel, info->device_label, DECKLINK_MAX_STRING_LENGTH);
                    SysFreeString(deviceLabel);
                }
                config->Release();
            }

            result = DECKLINK_OK;
            deckLink->Release();
            break;
        }

        currentIndex++;
        deckLink->Release();
    }

    iterator->Release();
    return result;
}

DeckLinkError decklink_get_api_version(char* version, int32_t max_length) {
    if (version == nullptr || max_length <= 0) {
        return DECKLINK_ERROR_INVALID_INDEX;
    }

    // Use the version from the header
    const char* api_version = BLACKMAGIC_DECKLINK_API_VERSION_STRING;
    int len = static_cast<int>(strlen(api_version));
    if (len >= max_length) {
        len = max_length - 1;
    }
    memcpy(version, api_version, len);
    version[len] = '\0';

    return DECKLINK_OK;
}

} // extern "C"

#else
// Non-Windows stub implementation
#include "decklink_wrapper.h"
#include <string.h>

extern "C" {

DeckLinkError decklink_init(void) {
    return DECKLINK_OK;
}

void decklink_cleanup(void) {
}

DeckLinkError decklink_get_device_count(int32_t* count) {
    if (count) *count = 0;
    return DECKLINK_ERROR_NO_DRIVER;
}

DeckLinkError decklink_get_device_info(int32_t index, DeckLinkDeviceInfo* info) {
    (void)index;
    (void)info;
    return DECKLINK_ERROR_NO_DRIVER;
}

DeckLinkError decklink_get_api_version(char* version, int32_t max_length) {
    if (version && max_length > 0) {
        const char* stub = "0.0.0 (stub)";
        strncpy(version, stub, max_length - 1);
        version[max_length - 1] = '\0';
    }
    return DECKLINK_OK;
}

} // extern "C"

#endif // _WIN32
