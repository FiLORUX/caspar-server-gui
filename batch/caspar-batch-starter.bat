@echo off
setlocal EnableExtensions EnableDelayedExpansion
chcp 65001 >nul

rem ---------------------------------------------------------------------------
rem CasparCG 2.5.0 launcher + config switcher (copy -> delay -> start minimised)
rem Lägg .bat-filen i samma mapp som:
rem   casparcg.exe, (ev) scanner.exe, och dina *.config-filer
rem ---------------------------------------------------------------------------

pushd "%~dp0"
title CasparCG 2.5.0 - Config Menu
mode con cols=72 lines=28
color 0B

rem === Mappa menyval -> configfil ===
set "CFG1=casparcg-1080p50-2ch.config"
set "CFG2=casparcg-1080i50-2ch.config"
set "CFG3=casparcg-1080p25-2ch.config"
set "CFG4=casparcg-1080p50-3ch.config"
set "CFG5=casparcg-1080i50-3ch.config"
set "CFG6=casparcg-1080p25-3ch.config"

:MENU
cls
echo ========================================================================
echo   CasparCG Launcher
echo ========================================================================
echo.
echo   [1] CasparCG Server 2.5.0 ^| 2 Channels ^| 1080p50
echo.
echo   [2] CasparCG Server 2.5.0 ^| 2 Channels ^| 1080i50
echo.
echo   [3] CasparCG Server 2.5.0 ^| 2 Channels ^| 1080p25
echo.
echo   [4] CasparCG Server 2.5.0 ^| 3 Channels ^| 1080p50
echo.
echo   [5] CasparCG Server 2.5.0 ^| 3 Channels ^| 1080i50
echo.
echo   [6] CasparCG Server 2.5.0 ^| 3 Channels ^| 1080p25
echo.
echo   [Q] Quit
echo.
choice /C 123456Q /N /M "Select: "

set "SEL=%ERRORLEVEL%"
if "%SEL%"=="7" goto :QUIT

set "PICKED=!CFG%SEL%!"
call :APPLY_CONFIG "!PICKED!"
goto :MENU


:APPLY_CONFIG
set "SRC=%~1"
set "DST=casparcg.config"

cls
echo ========================================================================
echo   Applying config
echo ========================================================================
echo.
echo   Source: %SRC%
echo   Dest:   %DST%
echo.

if not exist "%SRC%" (
  echo   ERROR: Missing file "%SRC%"
  echo.
  pause
  exit /b 1
)

copy /Y "%SRC%" "%DST%" >nul
if errorlevel 1 (
  echo   ERROR: Copy failed.
  echo.
  pause
  exit /b 1
)

echo   OK: Copied "%SRC%" -> "%DST%"
echo.
echo   Starting scanner.exe in 3s (if present; starts only if not running)...
echo   Starting casparcg.exe in 5s (minimised)...
echo.

rem 3s efter kopiering: scanner (minimised, och bara om den inte redan rullar)
timeout /t 3 /nobreak >nul
call :START_SCANNER_IF_NEEDED

rem Totalt 5s efter kopiering: caspar (minimised)
timeout /t 2 /nobreak >nul
call :CASPAR_LOOP

exit /b 0


:START_SCANNER_IF_NEEDED
if not exist "scanner.exe" exit /b 0

rem Kolla om scanner.exe redan kör
tasklist /FI "IMAGENAME eq scanner.exe" /NH 2>nul | find /I "scanner.exe" >nul
if not errorlevel 1 exit /b 0

start "" /min "%CD%\scanner.exe"
exit /b 0


:CASPAR_LOOP
if not exist "casparcg.exe" (
  echo   ERROR: casparcg.exe not found in "%CD%"
  echo.
  pause
  exit /b 1
)

:START
set ERRORLEVEL=0

rem Starta minimiserat och behåll exit code via PowerShell
call :RUN_CASPAR_MIN
if errorlevel 5 goto :START

exit /b 0


:RUN_CASPAR_MIN
powershell -NoProfile -ExecutionPolicy Bypass -Command ^
  "$p = Start-Process -FilePath '%CD%\casparcg.exe' -WindowStyle Minimized -PassThru -Wait; exit $p.ExitCode"
exit /b %ERRORLEVEL%


:QUIT
popd
endlocal
exit /b 0
