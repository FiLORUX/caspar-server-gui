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
#include <thread>
#include <atomic>
#include <map>
#include <mutex>

// Include the generated DeckLink API header (from IDL via MIDL)
#include "DeckLinkAPI_h.h"

// The compile-time SDK version constant lives in its own plain header, which the
// generated MIDL header does not pull in. Include it explicitly so the version
// helper below resolves BLACKMAGIC_DECKLINK_API_VERSION_STRING.
#include "DeckLinkAPIVersion.h"

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

/**
 * Convert a four-character code (as stored big-endian in a BMDDisplayMode) into
 * a printable C string, skipping any non-printable bytes. A zero code yields "".
 */
static void fourcc_to_cstr(uint32_t code, char* out, int max_length) {
    if (out == nullptr || max_length <= 0) {
        return;
    }
    int j = 0;
    for (int shift = 24; shift >= 0 && j < max_length - 1; shift -= 8) {
        char c = static_cast<char>((code >> shift) & 0xFF);
        if (c >= 32 && c < 127) {
            out[j++] = c;
        }
    }
    out[j] = '\0';
}

// RAII guard that guarantees COM is initialised on the calling thread for the
// duration of a single FFI call. Tauri dispatches each command on a tokio worker
// thread, and COM apartments are per-thread — a one-shot global CoInitialize on
// the first thread does not cover the others, so a later CoCreateInstance there
// would fail with CO_E_NOTINITIALIZED. Balancing init/uninit per call keeps every
// thread correct and the result deterministic regardless of which worker runs it.
namespace {
class ComApartment {
public:
    ComApartment() {
        HRESULT hr = CoInitializeEx(nullptr, COINIT_MULTITHREADED);
        // S_OK: we initialised it. S_FALSE: already initialised on this thread
        // (refcount bumped) — both must be balanced with CoUninitialize.
        // RPC_E_CHANGED_MODE: thread is already in another apartment (e.g. STA);
        // COM is usable and must NOT be torn down by us.
        ok_ = SUCCEEDED(hr) || hr == RPC_E_CHANGED_MODE;
        balance_ = (hr == S_OK || hr == S_FALSE);
    }
    ~ComApartment() {
        if (balance_) {
            CoUninitialize();
        }
    }
    bool ok() const { return ok_; }

    ComApartment(const ComApartment&) = delete;
    ComApartment& operator=(const ComApartment&) = delete;

private:
    bool ok_ = false;
    bool balance_ = false;
};
} // namespace

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
    if (count == nullptr) {
        return DECKLINK_ERROR_INVALID_INDEX;
    }

    *count = 0;

    ComApartment com;
    if (!com.ok()) {
        return DECKLINK_ERROR_COM_FAILED;
    }

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
    if (info == nullptr || index < 0) {
        return DECKLINK_ERROR_INVALID_INDEX;
    }

    // Zero out the structure
    memset(info, 0, sizeof(DeckLinkDeviceInfo));
    info->index = index;
    info->persistent_id = -1;
    info->device_group_id = -1;

    ComApartment com;
    if (!com.ok()) {
        return DECKLINK_ERROR_COM_FAILED;
    }

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
    version[0] = '\0';

    // Prefer the live version reported by the installed Desktop Video runtime.
    // The compile-time SDK constant only tells us which headers we built against;
    // it can differ from the driver actually loaded (observed: SDK 15.3 headers
    // against a 16.0.1 runtime), so the runtime value is the one worth surfacing.
    {
        ComApartment com;
        IDeckLinkAPIInformation* apiInfo = nullptr;
        HRESULT hr = com.ok()
            ? CoCreateInstance(
                  CLSID_CDeckLinkAPIInformation,
                  nullptr,
                  CLSCTX_ALL,
                  IID_IDeckLinkAPIInformation,
                  reinterpret_cast<void**>(&apiInfo))
            : E_FAIL;

        if (SUCCEEDED(hr) && apiInfo) {
            BSTR runtimeVersion = nullptr;
            if (apiInfo->GetString(BMDDeckLinkAPIVersion, &runtimeVersion) == S_OK && runtimeVersion) {
                bstr_to_cstr(runtimeVersion, version, max_length);
                SysFreeString(runtimeVersion);
            }
            apiInfo->Release();

            if (version[0] != '\0') {
                return DECKLINK_OK;
            }
        }
    }

    // Fall back to the compile-time SDK version if the runtime query is unavailable.
    const char* api_version = BLACKMAGIC_DECKLINK_API_VERSION_STRING;
    int len = static_cast<int>(strlen(api_version));
    if (len >= max_length) {
        len = max_length - 1;
    }
    memcpy(version, api_version, len);
    version[len] = '\0';

    return DECKLINK_OK;
}

DeckLinkError decklink_get_device_status(int32_t index, DeckLinkStatusInfo* status) {
    if (status == nullptr || index < 0) {
        return DECKLINK_ERROR_INVALID_INDEX;
    }

    memset(status, 0, sizeof(DeckLinkStatusInfo));

    ComApartment com;
    if (!com.ok()) {
        return DECKLINK_ERROR_COM_FAILED;
    }

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
            IDeckLinkStatus* deviceStatus = nullptr;
            if (deckLink->QueryInterface(IID_IDeckLinkStatus, reinterpret_cast<void**>(&deviceStatus)) == S_OK && deviceStatus) {
                // IDeckLinkStatus is a passive interface: it reports the current
                // signal state without opening the input, so polling it is cheap.
                BOOL inputLocked = FALSE;
                if (deviceStatus->GetFlag(bmdDeckLinkStatusVideoInputSignalLocked, &inputLocked) == S_OK) {
                    status->input_signal_locked = inputLocked != FALSE;
                }

                int64_t inputMode = 0;
                if (deviceStatus->GetInt(bmdDeckLinkStatusCurrentVideoInputMode, &inputMode) == S_OK) {
                    fourcc_to_cstr(static_cast<uint32_t>(inputMode), status->input_display_mode, sizeof(status->input_display_mode));
                }

                BOOL referenceLocked = FALSE;
                if (deviceStatus->GetFlag(bmdDeckLinkStatusReferenceSignalLocked, &referenceLocked) == S_OK) {
                    status->reference_signal_locked = referenceLocked != FALSE;
                }

                int64_t referenceMode = 0;
                if (deviceStatus->GetInt(bmdDeckLinkStatusReferenceSignalMode, &referenceMode) == S_OK) {
                    fourcc_to_cstr(static_cast<uint32_t>(referenceMode), status->reference_display_mode, sizeof(status->reference_display_mode));
                }

                deviceStatus->Release();
                result = DECKLINK_OK;
            } else {
                result = DECKLINK_ERROR_QUERY_FAILED;
            }

            deckLink->Release();
            break;
        }

        currentIndex++;
        deckLink->Release();
    }

    iterator->Release();
    return result;
}

DeckLinkError decklink_set_device_label(int32_t index, const char* label) {
    if (label == nullptr || index < 0) {
        return DECKLINK_ERROR_INVALID_INDEX;
    }

    ComApartment com;
    if (!com.ok()) {
        return DECKLINK_ERROR_COM_FAILED;
    }

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
            IDeckLinkConfiguration* config = nullptr;
            if (deckLink->QueryInterface(IID_IDeckLinkConfiguration, reinterpret_cast<void**>(&config)) == S_OK && config) {
                // Convert the UTF-8 label to a wide BSTR for the COM string setter.
                int wlen = MultiByteToWideChar(CP_UTF8, 0, label, -1, nullptr, 0);
                if (wlen > 0) {
                    std::wstring wide(static_cast<size_t>(wlen), L'\0');
                    MultiByteToWideChar(CP_UTF8, 0, label, -1, &wide[0], wlen);
                    BSTR labelBstr = SysAllocString(wide.c_str());
                    if (labelBstr != nullptr) {
                        HRESULT setHr = config->SetString(bmdDeckLinkConfigDeviceInformationLabel, labelBstr);
                        SysFreeString(labelBstr);
                        // Commit to NVRAM so it persists and is visible to other apps.
                        if (SUCCEEDED(setHr) && SUCCEEDED(config->WriteConfigurationToPreferences())) {
                            result = DECKLINK_OK;
                        } else {
                            result = DECKLINK_ERROR_QUERY_FAILED;
                        }
                    } else {
                        result = DECKLINK_ERROR_QUERY_FAILED;
                    }
                } else {
                    result = DECKLINK_ERROR_QUERY_FAILED;
                }
                config->Release();
            } else {
                result = DECKLINK_ERROR_QUERY_FAILED;
            }
            deckLink->Release();
            break;
        }
        currentIndex++;
        deckLink->Release();
    }

    iterator->Release();
    return result;
}

// ───────────────────────────────────────────────────────────────────────────
// Direct SDI output test
//
// Drives a card's SDI output directly via IDeckLinkOutput, bypassing CasparCG's
// GPU mixer (which renders black on some GPUs, e.g. AMD). Each device runs a
// background thread that holds one frame — a per-device colour with the device
// number drawn as large digits — on the output via DisplayVideoFrameSync.
// ───────────────────────────────────────────────────────────────────────────

// Internal-linkage state (static, so the surrounding extern "C" linkage does not
// apply); a namespace is not permitted inside an extern "C" block.
struct OutputTest {
    std::thread       worker;
    std::atomic<int>  state{0};   // 0 = starting, 1 = running, 2 = failed
    std::atomic<bool> stop{false};
};

static std::map<int, OutputTest*> g_output_tests;
static std::mutex                 g_output_tests_mutex;

// 3x5 bitmap font for digits 0-9 (each row uses the low 3 bits, MSB = left).
static const unsigned char DIGIT_FONT[10][5] = {
    {0b111, 0b101, 0b101, 0b101, 0b111}, // 0
    {0b010, 0b110, 0b010, 0b010, 0b111}, // 1
    {0b111, 0b001, 0b111, 0b100, 0b111}, // 2
    {0b111, 0b001, 0b111, 0b001, 0b111}, // 3
    {0b101, 0b101, 0b111, 0b001, 0b001}, // 4
    {0b111, 0b100, 0b111, 0b001, 0b111}, // 5
    {0b111, 0b100, 0b111, 0b101, 0b111}, // 6
    {0b111, 0b001, 0b010, 0b010, 0b010}, // 7
    {0b111, 0b101, 0b111, 0b101, 0b111}, // 8
    {0b111, 0b101, 0b111, 0b001, 0b111}, // 9
};

struct YuvColour { unsigned char y, u, v; };

// Distinct, well-separated BT.601 colours, one per device index.
static const YuvColour DEVICE_COLOURS[8] = {
    { 81,  90, 240}, // red
    {145,  54,  34}, // green
    { 41, 240, 110}, // blue
    {210,  16, 146}, // yellow
    {106, 202, 222}, // magenta
    {170, 166,  16}, // cyan
    {148,  73, 197}, // orange-ish
    {128, 128, 128}, // grey
};

// Render a per-device colour field with the 1-based device number as large white
// digits into an 8-bit UYVY buffer (w*2 bytes per row).
static void fill_test_frame(unsigned char* p, int w, int h, int index) {
    const YuvColour bg = DEVICE_COLOURS[index % 8];
    const int number = index + 1;

    char digits[4];
    int ndigits = 0;
    if (number < 100) {
        if (number >= 10) digits[ndigits++] = static_cast<char>('0' + (number / 10));
        digits[ndigits++] = static_cast<char>('0' + (number % 10));
    }

    const int cell = h / 7;                 // ~154 px per font pixel at 1080
    const int glyphW = 3 * cell;
    const int glyphH = 5 * cell;
    const int gap = cell;
    const int totalW = ndigits > 0 ? ndigits * glyphW + (ndigits - 1) * gap : 0;
    const int startX = (w - totalW) / 2;
    const int startY = (h - glyphH) / 2;

    auto isInk = [&](int x, int y) -> bool {
        if (ndigits == 0) return false;
        if (y < startY || y >= startY + glyphH) return false;
        int gy = (y - startY) / cell;       // 0..4
        if (gy < 0 || gy > 4) return false;
        for (int d = 0; d < ndigits; ++d) {
            int gx0 = startX + d * (glyphW + gap);
            if (x < gx0 || x >= gx0 + glyphW) continue;
            int gx = (x - gx0) / cell;       // 0..2
            if (gx < 0 || gx > 2) return false;
            int bit = (DIGIT_FONT[digits[d] - '0'][gy] >> (2 - gx)) & 1;
            return bit != 0;
        }
        return false;
    };

    for (int y = 0; y < h; ++y) {
        unsigned char* row = p + static_cast<size_t>(y) * w * 2;
        for (int xpair = 0; xpair < w / 2; ++xpair) {
            int x0 = xpair * 2;
            bool ink0 = isInk(x0, y);
            bool ink1 = isInk(x0 + 1, y);
            unsigned char* g = row + static_cast<size_t>(xpair) * 4;
            g[0] = (ink0 || ink1) ? 128 : bg.u;        // U (shared)
            g[1] = ink0 ? 235 : bg.y;                  // Y0
            g[2] = (ink0 || ink1) ? 128 : bg.v;        // V (shared)
            g[3] = ink1 ? 235 : bg.y;                  // Y1
        }
    }
}

static IDeckLink* get_device_at(int index) {
    IDeckLinkIterator* it = nullptr;
    if (FAILED(CoCreateInstance(CLSID_CDeckLinkIterator, nullptr, CLSCTX_ALL,
                                IID_IDeckLinkIterator, reinterpret_cast<void**>(&it))) || !it) {
        return nullptr;
    }
    IDeckLink* dl = nullptr;
    int cur = 0;
    while (it->Next(&dl) == S_OK) {
        if (cur == index) { it->Release(); return dl; }
        cur++;
        dl->Release();
    }
    it->Release();
    return nullptr;
}

// Runs on its own thread: owns all COM objects for the lifetime of the test so
// there is no cross-apartment sharing.
static void output_test_worker(int index, OutputTest* st) {
    CoInitializeEx(nullptr, COINIT_MULTITHREADED);

    IDeckLink* dl = get_device_at(index);
    IDeckLinkOutput* out = nullptr;
    if (dl) {
        dl->QueryInterface(IID_IDeckLinkOutput, reinterpret_cast<void**>(&out));
    }

    bool enabled = false;
    if (out) {
        const BMDDisplayMode modes[] = {
            bmdModeHD1080i5994, bmdModeHD1080p5994, bmdModeHD1080i50,
            bmdModeHD1080p25, bmdModeHD1080i6000, bmdModeHD1080p30,
        };
        for (BMDDisplayMode m : modes) {
            if (out->EnableVideoOutput(m, bmdVideoOutputFlagDefault) == S_OK) {
                enabled = true;
                break;
            }
        }
    }

    IDeckLinkMutableVideoFrame* frame = nullptr;
    if (enabled) {
        out->CreateVideoFrame(1920, 1080, 1920 * 2, bmdFormat8BitYUV,
                              bmdFrameFlagDefault, &frame);
    }
    if (frame) {
        IDeckLinkVideoBuffer* buf = nullptr;
        if (frame->QueryInterface(IID_IDeckLinkVideoBuffer, reinterpret_cast<void**>(&buf)) == S_OK && buf) {
            buf->StartAccess(bmdBufferAccessWrite);
            void* bytes = nullptr;
            buf->GetBytes(&bytes);
            if (bytes) {
                fill_test_frame(static_cast<unsigned char*>(bytes), 1920, 1080, index);
            }
            buf->EndAccess(bmdBufferAccessWrite);
            buf->Release();
        }
    }

    if (!frame) {
        st->state = 2;
        if (out) {
            if (enabled) out->DisableVideoOutput();
            out->Release();
        }
        if (dl) dl->Release();
        CoUninitialize();
        return;
    }

    st->state = 1;
    while (!st->stop.load()) {
        out->DisplayVideoFrameSync(frame);
    }

    frame->Release();
    out->DisableVideoOutput();
    out->Release();
    dl->Release();
    CoUninitialize();
}

DeckLinkError decklink_output_test_start(int32_t index) {
    if (index < 0) {
        return DECKLINK_ERROR_INVALID_INDEX;
    }
    OutputTest* st = nullptr;
    {
        std::lock_guard<std::mutex> lock(g_output_tests_mutex);
        if (g_output_tests.count(index)) {
            return DECKLINK_OK; // already running
        }
        st = new OutputTest();
        g_output_tests[index] = st;
        st->worker = std::thread(output_test_worker, static_cast<int>(index), st);
    }

    // Wait for the worker to report whether it actually got the SDI output open
    // (it fails if the card is busy — e.g. CasparCG is running and holds it).
    for (int i = 0; i < 150 && st->state.load() == 0; ++i) {
        Sleep(10);
    }
    if (st->state.load() == 2) {
        {
            std::lock_guard<std::mutex> lock(g_output_tests_mutex);
            auto it = g_output_tests.find(index);
            if (it != g_output_tests.end() && it->second == st) {
                g_output_tests.erase(it);
            }
        }
        st->stop = true;
        if (st->worker.joinable()) {
            st->worker.join();
        }
        delete st;
        return DECKLINK_ERROR_QUERY_FAILED;
    }
    return DECKLINK_OK;
}

DeckLinkError decklink_output_test_stop(int32_t index) {
    OutputTest* st = nullptr;
    {
        std::lock_guard<std::mutex> lock(g_output_tests_mutex);
        auto it = g_output_tests.find(index);
        if (it == g_output_tests.end()) {
            return DECKLINK_OK;
        }
        st = it->second;
        g_output_tests.erase(it);
    }
    st->stop = true;
    if (st->worker.joinable()) {
        st->worker.join();
    }
    delete st;
    return DECKLINK_OK;
}

void decklink_output_test_stop_all(void) {
    std::map<int, OutputTest*> taken;
    {
        std::lock_guard<std::mutex> lock(g_output_tests_mutex);
        taken.swap(g_output_tests);
    }
    for (auto& kv : taken) {
        kv.second->stop = true;
        if (kv.second->worker.joinable()) {
            kv.second->worker.join();
        }
        delete kv.second;
    }
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

DeckLinkError decklink_get_device_status(int32_t index, DeckLinkStatusInfo* status) {
    (void)index;
    if (status) {
        memset(status, 0, sizeof(DeckLinkStatusInfo));
    }
    return DECKLINK_ERROR_NO_DRIVER;
}

DeckLinkError decklink_set_device_label(int32_t index, const char* label) {
    (void)index;
    (void)label;
    return DECKLINK_ERROR_NO_DRIVER;
}

DeckLinkError decklink_output_test_start(int32_t index) {
    (void)index;
    return DECKLINK_ERROR_NO_DRIVER;
}

DeckLinkError decklink_output_test_stop(int32_t index) {
    (void)index;
    return DECKLINK_ERROR_NO_DRIVER;
}

void decklink_output_test_stop_all(void) {
}

} // extern "C"

#endif // _WIN32
