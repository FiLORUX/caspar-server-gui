@echo off
setlocal EnableExtensions EnableDelayedExpansion

rem ===========================================================================
rem  Thast - CasparCG 2.5.0 Dynamic Config Menu (Windows CMD)
rem
rem  Automatically discovers all *.config files in the current directory
rem  (excluding casparcg.config) and presents them as menu options.
rem
rem  Encoding note (for "Thåst" to display correctly):
rem  - Save this .bat as ANSI / Windows-1252 (Atom: ISO-8859-1 / Windows-1252)
rem  - Keep chcp 1252 below
rem ===========================================================================

chcp 1252 >nul

set "ROOT=%~dp0"
pushd "%ROOT%"

title Thast Caspar Launcher Config Menu
mode con cols=120 lines=50
color 0B

rem ===========================================================================
rem  DISCOVER CONFIG FILES
rem ===========================================================================

set "COUNT=0"
set "CHOICE_CHARS="

for %%f in (*.config) do (
    if /I not "%%~nxf"=="casparcg.config" (
        if !COUNT! LSS 9 (
            set /a COUNT+=1
            set "CFG!COUNT!=%%~f"
            set "FNAME!COUNT!=%%~nf"

            rem Get file modification date/time
            for %%a in ("%%f") do set "FTIME!COUNT!=%%~ta"

            rem Build choice characters string
            set "CHOICE_CHARS=!CHOICE_CHARS!!COUNT!"
        )
    )
)

rem Add S and Q to choice characters
set "CHOICE_CHARS=%CHOICE_CHARS%SQ"

rem ===========================================================================
rem  MAIN MENU
rem ===========================================================================

:MENU
cls
call :PRINT_BANNER

echo ======================================================================== >con
echo   Thast CasparCG Launcher                                               >con
echo ======================================================================== >con
echo.                                                                        >con

if "%COUNT%"=="0" (
    echo   No config files found in this directory.                          >con
    echo   Place *.config files here ^(excluding casparcg.config^).          >con
    echo.                                                                    >con
    echo   [S] Start ^(keep current casparcg.config^)                        >con
    echo   [Q] Quit                                                          >con
    echo.                                                                    >con
    choice /C SQ /N /T 15 /D S /M "Select: " >con
    if "!ERRORLEVEL!"=="1" (
        call :START_NO_COPY
        goto :WAIT_QUIT
    )
    goto :QUIT
)

rem Display discovered configs
for /L %%i in (1,1,%COUNT%) do (
    call :FORMAT_DISPLAY %%i
    echo   [%%i] !DISPLAY%%i! >con
    echo.                                                                    >con
)

echo   [S] Start ^(no copy; keep current casparcg.config^)                   >con
echo   [Q] Quit                                                              >con
echo.                                                                        >con
echo   Auto-start in 15 seconds ^(defaults to [S]^)                          >con
echo.                                                                        >con

rem Dynamic choice based on discovered files
choice /C %CHOICE_CHARS% /N /T 15 /D S /M "Select: " >con

set "SEL=%ERRORLEVEL%"

rem Calculate positions: Q is last, S is second-to-last
set /a "POS_Q=%COUNT%+2"
set /a "POS_S=%COUNT%+1"

if "%SEL%"=="%POS_Q%" goto :QUIT

if "%SEL%"=="%POS_S%" (
    call :START_NO_COPY
    goto :WAIT_QUIT
)

rem Selection is a config file (1-9)
if %SEL% LEQ %COUNT% (
    set "PICKED=!CFG%SEL%!"
    call :APPLY_CONFIG "!PICKED!"
    goto :WAIT_QUIT
)

goto :MENU


rem ===========================================================================
rem  FORMAT DISPLAY - Parse filename into readable format
rem ===========================================================================

:FORMAT_DISPLAY
set "IDX=%1"
set "FN=!FNAME%IDX%!"
set "FT=!FTIME%IDX%!"

rem Track if we found known patterns
set "RESOLUTION="
set "CHANNELS="

rem Parse filename: expected format casparcg-{resolution}-{channels}ch
rem Example: casparcg-1080p50-2ch -> 1080p50, 2

rem Remove "casparcg-" prefix if present
set "PARSED=!FN!"
if /I "!PARSED:~0,9!"=="casparcg-" set "PARSED=!PARSED:~9!"

rem Try to extract resolution (look for common patterns)
for %%r in (2160p60 2160p50 2160p30 2160p25 2160p24 1080p60 1080p50 1080p30 1080p25 1080p24 1080i60 1080i50 720p60 720p50 720p30 720p25 PAL NTSC) do (
    echo !PARSED! | find /I "%%r" >nul 2>&1
    if not errorlevel 1 set "RESOLUTION=%%r"
)

rem Try to extract channel count (look for Xch pattern)
for %%c in (1 2 3 4 5 6 7 8) do (
    echo !PARSED! | find /I "%%cch" >nul 2>&1
    if not errorlevel 1 set "CHANNELS=%%c"
)

rem Format the timestamp (already in local format from %%~t)
set "TIMESTAMP=!FT!"

rem Build display string based on what we found
if defined RESOLUTION (
    rem Known pattern found - use structured format
    if defined CHANNELS (
        set "DISPLAY%IDX%=CasparCG Config ^| !RESOLUTION! ^| !CHANNELS! Channels ^| !TIMESTAMP!"
    ) else (
        set "DISPLAY%IDX%=CasparCG Config ^| !RESOLUTION! ^| !TIMESTAMP!"
    )
) else (
    rem Unknown pattern - convert filename: replace - and _ with " | "
    set "READABLE=!FN!"
    set "READABLE=!READABLE:-= ^| !"
    set "READABLE=!READABLE:_= ^| !"
    set "DISPLAY%IDX%=!READABLE! ^| !TIMESTAMP!"
)

exit /b 0


rem ===========================================================================
rem  PRINT BANNER
rem ===========================================================================

:PRINT_BANNER
echo(  ________      __       __     ______                            ____________    >con
echo( /_  __/ /_  __(())_____/ /_   / ____/___ __________  ____ ______/ ____/ ____/    >con
echo(   / / / __ \/ __ `/ ___/ __/  / /   / __ `/ ___/ __ \/ __ `/ ___/ /   / / __      >con
echo(  / / / / / / /_/ (__  ) /_   / /___/ /_/ (__  ) /_/ / /_/ / /  / /___/ /_/ /      >con
echo( /_/ /_/ /_/\__,_/____/\__/   \____/\__,_/____/ .___/\__,_/_/   \____/\____/       >con
echo(    _____                              __    /_/                    __             >con
echo(   / ___/___  ______   _____  _____   / /   ____ ___  ______  _____/ /_  ___  _____>con
echo(   \__ \/ _ \/ ___/ ^| / / _ \/ ___/  / /   / __ `/ / / / __ \/ ___/ __ \/ _ \/ ___/>con
echo(  ___/ /  __/ /   ^| ^|/ /  __/ /     / /___/ /_/ / /_/ / / / / /__/ / / /  __/ /    >con
echo( /____/\___/_/    ^|___/\___/_/     /_____/\__,_/\__,_/_/ /_/\___/_/ /_/\___/_/     >con
echo(                                                                                   >con
echo(>con
exit /b 0


rem ===========================================================================
rem  START WITHOUT COPYING CONFIG
rem ===========================================================================

:START_NO_COPY
cls
echo ======================================================================== >con
echo   Starting (no copy)                                                    >con
echo ======================================================================== >con
echo.                                                                        >con
echo   Keeping existing casparcg.config                                      >con
echo.                                                                        >con

call :START_ALL
exit /b 0


rem ===========================================================================
rem  APPLY SELECTED CONFIG
rem ===========================================================================

:APPLY_CONFIG
set "SRC=%~1"
set "DST=%ROOT%casparcg.config"

cls
echo ======================================================================== >con
echo   Applying config                                                       >con
echo ======================================================================== >con
echo.                                                                        >con
echo   Source: %~nx1                                                         >con
echo   Dest:   casparcg.config                                               >con
echo.                                                                        >con

if not exist "%SRC%" (
    echo   ERROR: Missing file "%SRC%"                                       >con
    echo.                                                                    >con
    pause >con
    exit /b 1
)

copy /Y "%SRC%" "%DST%" >nul 2>&1
if errorlevel 1 (
    echo   ERROR: Copy failed.                                               >con
    echo.                                                                    >con
    pause >con
    exit /b 1
)

echo   OK: Copied "%~nx1" -^> "casparcg.config"                              >con
echo.                                                                        >con

call :START_ALL
exit /b 0


rem ===========================================================================
rem  START CASPAR AND SCANNER
rem ===========================================================================

:START_ALL
echo   Starting scanner.exe in 3s (if present; only if not running)...       >con
echo   Starting casparcg.exe in 5s (minimised)...                            >con
echo.                                                                        >con

timeout /t 3 /nobreak >nul
call :START_SCANNER_IF_NEEDED

timeout /t 2 /nobreak >nul
call :START_CASPAR_SUPERVISOR_IF_NEEDED
exit /b 0


rem ===========================================================================
rem  START SCANNER (if present and not running)
rem ===========================================================================

:START_SCANNER_IF_NEEDED
if not exist "%ROOT%scanner.exe" exit /b 0

tasklist /FI "IMAGENAME eq scanner.exe" /NH 2>nul | find /I "scanner.exe" >nul
if not errorlevel 1 exit /b 0

start "" /min "%ROOT%scanner.exe"
exit /b 0


rem ===========================================================================
rem  START CASPARCG WITH SUPERVISOR (restart on exit code 5)
rem ===========================================================================

:START_CASPAR_SUPERVISOR_IF_NEEDED
if not exist "%ROOT%casparcg.exe" (
    echo   ERROR: casparcg.exe not found in "%ROOT%"                         >con
    echo.                                                                    >con
    pause >con
    exit /b 1
)

rem If CasparCG is already running, do not start another instance.
tasklist /FI "IMAGENAME eq casparcg.exe" /NH 2>nul | find /I "casparcg.exe" >nul
if not errorlevel 1 exit /b 0

set "CASPAR_EXE=%ROOT%casparcg.exe"
set "PSCMD=$exe='%CASPAR_EXE%'; while ($true) { $p = Start-Process -FilePath $exe -WindowStyle Minimized -PassThru -Wait; if ($p.ExitCode -ne 5) { break } }"

rem Run supervisor in the background (hidden) so the launcher can accept Q
start "" /min powershell -NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -Command "%PSCMD%"
exit /b 0


rem ===========================================================================
rem  WAIT FOR QUIT
rem ===========================================================================

:WAIT_QUIT
cls
call :PRINT_BANNER
echo   CasparCG started (minimised). Scanner started if present.             >con
echo(>con

choice /C Q /N /M "   Q: Quit launcher " >con
goto :QUIT


rem ===========================================================================
rem  QUIT
rem ===========================================================================

:QUIT
popd
endlocal
exit /b 0
