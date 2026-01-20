/**
 * DeckLink SDK C Wrapper for Rust FFI
 *
 * Provides a simple C API for enumerating DeckLink devices and querying their
 * capabilities. Designed for use with Rust FFI bindings.
 *
 * Copyright (c) 2026 Thast. MIT License.
 */

#ifndef DECKLINK_WRAPPER_H
#define DECKLINK_WRAPPER_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Maximum string length for device names and labels
 */
#define DECKLINK_MAX_STRING_LENGTH 256

/**
 * Video connection types (bitmask)
 */
typedef enum {
    DECKLINK_VIDEO_CONNECTION_SDI             = 1 << 0,
    DECKLINK_VIDEO_CONNECTION_HDMI            = 1 << 1,
    DECKLINK_VIDEO_CONNECTION_OPTICAL_SDI     = 1 << 2,
    DECKLINK_VIDEO_CONNECTION_COMPONENT       = 1 << 3,
    DECKLINK_VIDEO_CONNECTION_COMPOSITE       = 1 << 4,
    DECKLINK_VIDEO_CONNECTION_SVIDEO          = 1 << 5,
} DeckLinkVideoConnection;

/**
 * Audio connection types (bitmask)
 */
typedef enum {
    DECKLINK_AUDIO_CONNECTION_EMBEDDED        = 1 << 0,
    DECKLINK_AUDIO_CONNECTION_AESEBU          = 1 << 1,
    DECKLINK_AUDIO_CONNECTION_ANALOG          = 1 << 2,
    DECKLINK_AUDIO_CONNECTION_ANALOG_XLR      = 1 << 3,
    DECKLINK_AUDIO_CONNECTION_ANALOG_RCA      = 1 << 4,
    DECKLINK_AUDIO_CONNECTION_MICROPHONE      = 1 << 5,
    DECKLINK_AUDIO_CONNECTION_HEADPHONES      = 1 << 6,
} DeckLinkAudioConnection;

/**
 * Device IO support flags
 */
typedef enum {
    DECKLINK_IO_SUPPORT_CAPTURE               = 1 << 0,
    DECKLINK_IO_SUPPORT_PLAYBACK              = 1 << 1,
} DeckLinkIOSupport;

/**
 * Device information structure
 */
typedef struct {
    int32_t index;
    char display_name[DECKLINK_MAX_STRING_LENGTH];
    char model_name[DECKLINK_MAX_STRING_LENGTH];
    char device_label[DECKLINK_MAX_STRING_LENGTH];
    int64_t persistent_id;
    int64_t device_group_id;
    int32_t sub_device_index;
    int32_t num_sub_devices;
    uint32_t video_input_connections;
    uint32_t video_output_connections;
    uint32_t audio_input_connections;
    uint32_t audio_output_connections;
    uint32_t io_support;
    bool supports_internal_keying;
    bool supports_external_keying;
    bool supports_dual_link_sdi;
    bool supports_quad_link_sdi;
    bool supports_idle_output;
    int32_t max_audio_channels;
} DeckLinkDeviceInfo;

/**
 * Error codes
 */
typedef enum {
    DECKLINK_OK = 0,
    DECKLINK_ERROR_NOT_INITIALISED = -1,
    DECKLINK_ERROR_COM_FAILED = -2,
    DECKLINK_ERROR_NO_DRIVER = -3,
    DECKLINK_ERROR_INVALID_INDEX = -4,
    DECKLINK_ERROR_QUERY_FAILED = -5,
} DeckLinkError;

/**
 * Initialise the DeckLink wrapper (calls CoInitialize on Windows)
 *
 * @return DECKLINK_OK on success, error code otherwise
 */
DeckLinkError decklink_init(void);

/**
 * Clean up the DeckLink wrapper (calls CoUninitialize on Windows)
 */
void decklink_cleanup(void);

/**
 * Get the number of DeckLink devices in the system
 *
 * @param count Pointer to store the device count
 * @return DECKLINK_OK on success, error code otherwise
 */
DeckLinkError decklink_get_device_count(int32_t* count);

/**
 * Get information about a specific DeckLink device
 *
 * @param index Device index (0-based)
 * @param info Pointer to DeckLinkDeviceInfo structure to fill
 * @return DECKLINK_OK on success, error code otherwise
 */
DeckLinkError decklink_get_device_info(int32_t index, DeckLinkDeviceInfo* info);

/**
 * Get the DeckLink API version string
 *
 * @param version Buffer to store version string (at least 32 bytes)
 * @param max_length Maximum length of buffer
 * @return DECKLINK_OK on success
 */
DeckLinkError decklink_get_api_version(char* version, int32_t max_length);

#ifdef __cplusplus
}
#endif

#endif /* DECKLINK_WRAPPER_H */
