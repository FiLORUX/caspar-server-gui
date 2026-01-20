# CasparCG 2.5.0 – Dynamic Windows Batch Launcher

A Windows `.bat` menu that **automatically discovers** all `*.config` files in the current directory (excluding `casparcg.config`) and presents them as selectable options.

## Features

- **Dynamic discovery**: No hardcoded presets — finds all `*.config` files automatically
- **Smart parsing**: Extracts resolution and channel count from filenames
- **Modification timestamp**: Shows when each config was last modified
- **Up to 9 configs**: Limited by Windows `choice` command
- **Auto-start timeout**: Defaults to [S] after 15 seconds
- **Scanner support**: Starts `scanner.exe` if present (and not already running)
- **Supervisor mode**: Restarts CasparCG on exit code 5

## Menu Display Format

```
[1] CasparCG Config | 1080p50 | 2 Channels | 2026-01-20 14:32
[2] CasparCG Config | 1080i50 | 3 Channels | 2026-01-19 09:15
[3] CasparCG Config | 720p50 | 2026-01-18 16:45

[S] Start (no copy; keep current casparcg.config)
[Q] Quit
```

## Filename Convention

For best results, name your config files following this pattern:

```
casparcg-{resolution}-{channels}ch.config
```

Examples:
- `casparcg-1080p50-2ch.config` → "CasparCG Config | 1080p50 | 2 Channels | 2026-01-20 14:32"
- `casparcg-1080i50-3ch.config` → "CasparCG Config | 1080i50 | 3 Channels | 2026-01-19 09:15"
- `casparcg-720p25-2ch.config` → "CasparCG Config | 720p25 | 2 Channels | 2026-01-18 16:45"

### Fallback for Custom Filenames

If the filename doesn't match the expected pattern, it displays the filename with `-` and `_` replaced by ` | `:

- `my-custom-config.config` → "my | custom | config | 2026-01-20 10:00"
- `studio_a_backup.config` → "studio | a | backup | 2026-01-19 08:30"
- `test.config` → "test | 2026-01-18 12:00"

### Supported Resolutions (auto-detected)

- 2160p: 2160p60, 2160p50, 2160p30, 2160p25, 2160p24
- 1080p: 1080p60, 1080p50, 1080p30, 1080p25, 1080p24
- 1080i: 1080i60, 1080i50
- 720p: 720p60, 720p50, 720p30, 720p25
- SD: PAL, NTSC

### Supported Channel Counts

1–8 channels (detected from `Xch` pattern in filename)

## Folder Layout

Place the batch file in your CasparCG server directory:

```
CasparCG-Server/
├── casparcg.exe
├── scanner.exe (optional)
├── casparcg.config (active config, created/overwritten)
├── casparcg-1080p50-2ch.config
├── casparcg-1080i50-2ch.config
├── casparcg-720p50-3ch.config
├── ... (any other *.config files)
└── caspar-batch-starter.bat
```

## Usage

1. Double-click `caspar-batch-starter.bat`
2. The menu shows all discovered config files with parsed details
3. Press `1`–`9` to select a config (copies it to `casparcg.config`)
4. Press `S` to start without changing config
5. Press `Q` to quit
6. After 15 seconds of inactivity, defaults to `S`

## How It Works

1. **Discovery**: Scans for `*.config` files, excludes `casparcg.config`
2. **Parsing**: Extracts resolution/channels from filename using pattern matching
3. **Timestamp**: Reads file modification time via `%%~t` expansion
4. **Selection**: Uses `choice /C` with dynamically built character set
5. **Apply**: Copies selected config to `casparcg.config`
6. **Start**: Launches scanner (if present) then CasparCG (minimised)

## Limitations

- Maximum 9 config files (Windows `choice` limitation)
- Timestamp format depends on Windows regional settings

## Requirements

- Windows 10/11
- PowerShell (for supervisor/restart functionality)
- Standard Windows commands: `choice`, `timeout`, `tasklist`, `find`

## Encoding

Save the `.bat` file as **ANSI / Windows-1252** for the "Thåst" banner to display correctly.

## Licence

MIT – Free to use and modify.
