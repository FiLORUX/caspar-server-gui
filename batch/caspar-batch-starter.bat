@echo off
setlocal EnableExtensions EnableDelayedExpansion

rem ===========================================================================
rem  Thast - CasparCG 2.5.0 Config Menu (Windows CMD)
rem
rem  Encoding note (for "Thåst" to display correctly):
rem  - Save this .bat as ANSI / Windows-1252 (Atom: ISO-8859-1 / Windows-1252)
rem  - Keep chcp 1252 below
rem ===========================================================================

chcp 1252 >nul

set "ROOT=%~dp0"
pushd "%ROOT%"

title Thast Caspar Launcher Config Menu
mode con cols=120 lines=45
color 0B

rem === Menu -> config mapping (full paths) ===
set "CFG1=%ROOT%casparcg-1080p50-2ch.config"
set "CFG2=%ROOT%casparcg-1080i50-2ch.config"
set "CFG3=%ROOT%casparcg-1080p25-2ch.config"
set "CFG4=%ROOT%casparcg-1080p50-3ch.config"
set "CFG5=%ROOT%casparcg-1080i50-3ch.config"
set "CFG6=%ROOT%casparcg-1080p25-3ch.config"

:MENU
cls
call :PRINT_BANNER

echo ======================================================================== >con
echo   Thast CasparCG Launcher                                               >con
echo ======================================================================== >con
echo.                                                                       >con
echo   [1] CasparCG Server 2.5.0 ^| 2 Channels ^| 1080p50                     >con
echo.                                                                       >con
echo   [2] CasparCG Server 2.5.0 ^| 2 Channels ^| 1080i50                     >con
echo.                                                                       >con
echo   [3] CasparCG Server 2.5.0 ^| 2 Channels ^| 1080p25                     >con
echo.                                                                       >con
echo   [4] CasparCG Server 2.5.0 ^| 3 Channels ^| 1080p50                     >con
echo.                                                                       >con
echo   [5] CasparCG Server 2.5.0 ^| 3 Channels ^| 1080i50                     >con
echo.                                                                       >con
echo   [6] CasparCG Server 2.5.0 ^| 3 Channels ^| 1080p25                     >con
echo.                                                                       >con
echo   [S] Start (no copy; keep current casparcg.config)                     >con
echo   [Q] Quit                                                             >con
echo.                                                                       >con
echo   Auto-start in 15 seconds (defaults to [S])                            >con
echo.                                                                       >con

rem Default after 15s: S (start without copying any config)
choice /C 123456SQ /N /T 15 /D S /M "Select: " >con

set "SEL=%ERRORLEVEL%"

rem Q is 8th char in /C 123456SQ
if "%SEL%"=="8" goto :QUIT

rem S is 7th char
if "%SEL%"=="7" (
  call :START_NO_COPY
  goto :WAIT_QUIT
)

set "PICKED=!CFG%SEL%!"
call :APPLY_CONFIG "!PICKED!"
goto :WAIT_QUIT


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


:START_NO_COPY
cls
echo ======================================================================== >con
echo   Starting (no copy)                                                    >con
echo ======================================================================== >con
echo.                                                                       >con
echo   Keeping existing casparcg.config                                      >con
echo.                                                                       >con

call :START_ALL
exit /b 0


:APPLY_CONFIG
set "SRC=%~1"
set "DST=%ROOT%casparcg.config"

cls
echo ======================================================================== >con
echo   Applying config                                                       >con
echo ======================================================================== >con
echo.                                                                       >con
echo   Source: %~nx1                                                         >con
echo   Dest:   casparcg.config                                               >con
echo.                                                                       >con

if not exist "%SRC%" (
  echo   ERROR: Missing file "%SRC%"                                         >con
  echo.                                                                      >con
  pause >con
  exit /b 1
)

copy /Y "%SRC%" "%DST%" >nul 2>&1
if errorlevel 1 (
  echo   ERROR: Copy failed.                                                 >con
  echo.                                                                      >con
  pause >con
  exit /b 1
)

echo   OK: Copied "%~nx1" -> "casparcg.config"                                >con
echo.                                                                       >con

call :START_ALL
exit /b 0


:START_ALL
echo   Starting scanner.exe in 3s (if present; only if not running)...       >con
echo   Starting casparcg.exe in 5s (minimised)...                             >con
echo.                                                                       >con

timeout /t 3 /nobreak >nul
call :START_SCANNER_IF_NEEDED

timeout /t 2 /nobreak >nul
call :START_CASPAR_SUPERVISOR_IF_NEEDED
exit /b 0


:START_SCANNER_IF_NEEDED
if not exist "%ROOT%scanner.exe" exit /b 0

tasklist /FI "IMAGENAME eq scanner.exe" /NH 2>nul | find /I "scanner.exe" >nul
if not errorlevel 1 exit /b 0

start "" /min "%ROOT%scanner.exe"
exit /b 0


:START_CASPAR_SUPERVISOR_IF_NEEDED
if not exist "%ROOT%casparcg.exe" (
  echo   ERROR: casparcg.exe not found in "%ROOT%"                           >con
  echo.                                                                      >con
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


:WAIT_QUIT
cls
call :PRINT_BANNER
echo   CasparCG started (minimised). Scanner started if present.              >con
echo(>con

choice /C Q /N /M "   Q: Quit launcher " >con
goto :QUIT


:QUIT
popd
endlocal
exit /b 0
