# CasparCG 2.5.0 – Windows Batch Launcher (Config Menu)

A Windows `.bat` menu that switches between multiple CasparCG configurations by copying a selected `casparcg-*.config` to `casparcg.config` (overwrite without prompting), then starts:

- `scanner.exe` **(if present)** after **3 seconds** (minimised, and only if it is not already running)
- `casparcg.exe` after **5 seconds** (minimised), with a loop that restarts CasparCG if the exit code is `5`

## Features

- Console menu with 6 presets:
  - 2 Channels: 1080p50 / 1080i50 / 1080p25
  - 3 Channels: 1080p50 / 1080i50 / 1080p25
- Copies preset config → `casparcg.config` using `copy /Y`
- Starts `scanner.exe` minimised if the file exists and the process is not already running
- Starts `casparcg.exe` minimised and waits for it to exit (via PowerShell) to reliably capture the exit code
- Loop: if CasparCG exits with code `5`, it is restarted

## Folder layout

Place everything in the same directory:

your-folder
casparcg.exe
scanner.exe (optional)
casparcg.config (created/overwritten)
casparcg-1080p50-2ch.config
casparcg-1080i50-2ch.config
casparcg-1080p25-2ch.config
casparcg-1080p50-3ch.config
casparcg-1080i50-3ch.config
casparcg-1080p25-3ch.config
caspar-batch-starter.bat


> The batch uses `%~dp0` and runs from its own directory, so it is relatively portable.

## Usage

1. Run `casparcg-menu.bat` (double-click or from Command Prompt).
2. Choose `1–6` for the required format/channel setup.
3. The batch copies the chosen config to `casparcg.config`.
4. After 3 seconds, `scanner.exe` starts (if present).
5. After a total of 5 seconds, `casparcg.exe` starts.

To quit the menu, press `Q`.

## Presets mapping

| Menu | Label | Copied to `casparcg.config` |
|---:|---|---|
| 1 | 2 Channels / 1080p50 | `casparcg-1080p50-2ch.config` |
| 2 | 2 Channels / 1080i50 | `casparcg-1080i50-2ch.config` |
| 3 | 2 Channels / 1080p25 | `casparcg-1080p25-2ch.config` |
| 4 | 3 Channels / 1080p50 | `casparcg-1080p50-3ch.config` |
| 5 | 3 Channels / 1080i50 | `casparcg-1080i50-3ch.config` |
| 6 | 3 Channels / 1080p25 | `casparcg-1080p25-3ch.config` |

## Notes on “minimised” and exit codes

- `scanner.exe` is launched using `start "" /min ...`.
- `casparcg.exe` is launched via PowerShell `Start-Process ... -Wait -PassThru` to:
  - start minimised
  - capture the process exit code reliably
- Exit code `5` triggers a restart (`goto :START`). Any other exit code ends the loop.

## Requirements

- Windows `cmd.exe`
- PowerShell available (standard on modern Windows)
- Built-in tools: `timeout`, `tasklist`, `find`

## Customisation

### Add or change presets
Edit `CFG1..CFG6` and the menu text. If adding more, also update the `choice /C ...` set.

Example:

```bat
set "CFG7=casparcg-720p50-2ch.config"


Then add 7 to choice /C 1234567Q and add a menu line.

Change delays

Delays are controlled by:

timeout /t 3 /nobreak (scanner)

timeout /t 2 /nobreak (additional 2 seconds → total 5 seconds before CasparCG)

Troubleshooting

“Missing file”: the selected casparcg-*.config does not exist in the directory.

CasparCG does not start: confirm casparcg.exe is in the same folder as the batch.

scanner does not start: confirm scanner.exe is named exactly and is not already running (the script checks via tasklist).

Licence

Internal use: unrestricted. If publishing in a repository, consider adding an MIT licence (or your preferred equivalent).
