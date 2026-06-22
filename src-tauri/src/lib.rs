// CasparCG Server GUI - Rust Backend
// Provides configuration management, AMCP communication, and DeckLink integration

mod amcp;
mod config;
mod decklink;
mod http_server;
mod system;
mod tsl;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{Emitter, Manager};
use tokio::sync::Mutex;

use config::{
    generate_caspar_xml, parse_caspar_xml, CasparConfig, GlobalConfig, GuiSettings, MediaServer,
};
use decklink::{DeckLinkDevice, DeckLinkStatus, DuplexMode};

// Public re-exports for hardware-in-the-loop tests and external tooling. These
// expose the same enumeration path the Tauri commands use, without making the
// whole module public.
pub use decklink::{
    get_api_version as decklink_api_version, get_device_status as decklink_device_status,
    list_devices as enumerate_decklink_devices, set_device_label as set_decklink_device_label,
};

/// Application state shared across commands
pub struct AppState {
    pub amcp_client: Arc<Mutex<amcp::AmcpClient>>,
    pub gui_settings: Arc<Mutex<GuiSettings>>,
    pub test_server: http_server::TestServerState,
    pub tsl_monitor: tsl::TslState,
    /// The launched CasparCG server process, if running
    pub caspar_process: Arc<Mutex<Option<std::process::Child>>>,
    /// The media scanner process launched alongside the server, if running
    pub scanner_process: Arc<Mutex<Option<std::process::Child>>>,
    /// Desired state: true while the server should be supervised. Start sets it,
    /// Stop/app-exit/give-up clear it. The supervisor consults it so a user Stop
    /// is never mistaken for a crash and never triggers a restart.
    pub server_should_run: Arc<AtomicBool>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            amcp_client: Arc::new(Mutex::new(amcp::AmcpClient::new())),
            gui_settings: Arc::new(Mutex::new(GuiSettings::load())),
            test_server: http_server::create_test_server_state(),
            tsl_monitor: tsl::create_tsl_state(),
            caspar_process: Arc::new(Mutex::new(None)),
            scanner_process: Arc::new(Mutex::new(None)),
            server_should_run: Arc::new(AtomicBool::new(false)),
        }
    }
}

// ============================================================================
// Configuration Commands
// ============================================================================

/// Load a CasparCG XML configuration file
#[tauri::command]
async fn load_caspar_config(path: String) -> Result<CasparConfig, String> {
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    parse_caspar_xml(&content).map_err(|e| format!("Failed to parse config: {}", e))
}

/// Save a CasparCG XML configuration file
#[tauri::command]
async fn save_caspar_config(path: String, config: CasparConfig) -> Result<(), String> {
    let xml = generate_caspar_xml(&config)
        .map_err(|e| format!("Failed to generate XML: {}", e))?;

    std::fs::write(&path, xml).map_err(|e| format!("Failed to write file: {}", e))
}

/// Load a global configuration profile (JSON)
#[tauri::command]
async fn load_global_config(path: String) -> Result<GlobalConfig, String> {
    GlobalConfig::load_from_file(&PathBuf::from(&path))
        .map_err(|e| format!("Failed to load config: {}", e))
}

/// Save a global configuration profile (JSON)
#[tauri::command]
async fn save_global_config(path: String, mut config: GlobalConfig) -> Result<(), String> {
    config.touch();
    config
        .save_to_file(&PathBuf::from(&path))
        .map_err(|e| format!("Failed to save config: {}", e))
}

/// Export global config to CasparCG XML format
#[tauri::command]
async fn export_to_caspar_xml(config: GlobalConfig) -> Result<String, String> {
    generate_caspar_xml(&config.caspar).map_err(|e| format!("Failed to generate XML: {}", e))
}

/// Create a new global config with default values
#[tauri::command]
async fn create_default_config(name: String) -> Result<GlobalConfig, String> {
    Ok(GlobalConfig::new(name))
}

/// Get list of available profiles in the profiles directory
#[tauri::command]
async fn list_profiles(state: tauri::State<'_, AppState>) -> Result<Vec<String>, String> {
    let settings = state.gui_settings.lock().await;
    let profiles_dir = settings
        .profiles_dir()
        .ok_or_else(|| "CasparCG path not set".to_string())?;

    if !profiles_dir.exists() {
        return Ok(vec![]);
    }

    let entries = std::fs::read_dir(&profiles_dir)
        .map_err(|e| format!("Failed to read profiles directory: {}", e))?;

    let profiles: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "json")
                .unwrap_or(false)
        })
        .filter_map(|e| {
            e.path()
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
        })
        .collect();

    Ok(profiles)
}

// ============================================================================
// DeckLink Commands
// ============================================================================

/// List all available DeckLink devices
#[tauri::command]
async fn list_decklink_devices() -> Result<Vec<DeckLinkDevice>, String> {
    decklink::list_devices().map_err(|e| e.to_string())
}

/// Get information about a specific DeckLink device
#[tauri::command]
async fn get_decklink_info(persistent_id: String) -> Result<DeckLinkDevice, String> {
    decklink::get_device_by_id(&persistent_id).map_err(|e| e.to_string())
}

/// Write a persistent display label to the DeckLink device's NVRAM
#[tauri::command]
async fn set_decklink_label(
    persistent_id: String,
    label: String,
) -> Result<(), String> {
    decklink::set_device_label(&persistent_id, &label).map_err(|e| e.to_string())
}

/// Set the duplex mode for a DeckLink device
#[tauri::command]
async fn set_decklink_duplex_mode(
    persistent_id: String,
    mode: String,
) -> Result<(), String> {
    let duplex_mode: DuplexMode = mode.parse().map_err(|e: decklink::DeckLinkError| e.to_string())?;
    decklink::set_duplex_mode(&persistent_id, duplex_mode).map_err(|e| e.to_string())
}

/// Get DeckLink driver version
#[tauri::command]
async fn get_decklink_driver_version() -> Result<Option<String>, String> {
    decklink::get_driver_version().map_err(|e| e.to_string())
}

/// Get live signal status for a DeckLink device (by 1-based device index)
#[tauri::command]
async fn get_decklink_status(index: u32) -> Result<DeckLinkStatus, String> {
    decklink::get_device_status(index).map_err(|e| e.to_string())
}

/// Start a direct SDI output test on a device (1-based index). Drives the SDI
/// output directly via the DeckLink SDK, bypassing CasparCG's GPU mixer, so the
/// physical output can be verified even where CasparCG renders black.
#[tauri::command]
async fn start_decklink_output_test(index: u32, mode: u32) -> Result<(), String> {
    decklink::output_test_start(index, mode).map_err(|e| e.to_string())
}

/// Stop a direct SDI output test on a device (1-based index).
#[tauri::command]
async fn stop_decklink_output_test(index: u32) -> Result<(), String> {
    decklink::output_test_stop(index).map_err(|e| e.to_string())
}

// ============================================================================
// AMCP Commands
// ============================================================================

/// Connect to CasparCG server via AMCP
#[tauri::command]
async fn amcp_connect(
    host: String,
    port: u16,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut client = state.amcp_client.lock().await;
    client
        .connect(&host, port)
        .await
        .map_err(|e| e.to_string())?;

    // Update GUI settings
    let mut settings = state.gui_settings.lock().await;
    settings.last_host = Some(host);
    settings.last_port = Some(port);
    settings.last_server_was_running = true;
    let _ = settings.save();

    Ok(())
}

/// Disconnect from CasparCG server
#[tauri::command]
async fn amcp_disconnect(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut client = state.amcp_client.lock().await;
    client.disconnect().await;

    // Update GUI settings
    let mut settings = state.gui_settings.lock().await;
    settings.last_server_was_running = false;
    let _ = settings.save();

    Ok(())
}

/// Check if connected to AMCP server
#[tauri::command]
async fn amcp_is_connected(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    let client = state.amcp_client.lock().await;
    Ok(client.is_connected())
}

/// Get connection info
#[tauri::command]
async fn amcp_connection_info(
    state: tauri::State<'_, AppState>,
) -> Result<Option<(String, u16)>, String> {
    let client = state.amcp_client.lock().await;
    Ok(client.connection_info())
}

/// Get CasparCG server version
#[tauri::command]
async fn amcp_version(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let client = state.amcp_client.lock().await;
    client.version().await.map_err(|e| e.to_string())
}

/// Get system information from server
#[tauri::command]
async fn amcp_info_system(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let client = state.amcp_client.lock().await;
    client.info_system().await.map_err(|e| e.to_string())
}

/// Send raw AMCP command
#[tauri::command]
async fn amcp_send_command(
    command: String,
    state: tauri::State<'_, AppState>,
) -> Result<amcp::AmcpResponse, String> {
    let client = state.amcp_client.lock().await;
    client.send_command(&command).await.map_err(|e| e.to_string())
}

// ============================================================================
// Test Server Commands
// ============================================================================

/// Start the test HTTP server for serving test patterns to CasparCG
///
/// Returns the port the server is running on.
#[tauri::command]
async fn start_test_server(
    port: Option<u16>,
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<u16, String> {
    // Locate the bundled key-fill-identifier assets. Tauri rewrites the "../" in
    // a resource path to "_up_", so in an installed build the folder is at
    // <resource_dir>/_up_/key-fill-identifier, not directly under the resource
    // dir. Probe every plausible location (resources, next to the exe, dev tree)
    // and pick the first that actually contains index.html.
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(res) = app.path().resource_dir() {
        candidates.push(res.join("_up_").join("key-fill-identifier"));
        candidates.push(res.join("key-fill-identifier"));
        candidates.push(res);
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join("key-fill-identifier"));
            candidates.push(dir.join("resources").join("key-fill-identifier"));
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("key-fill-identifier"));
        if let Some(parent) = cwd.parent() {
            candidates.push(parent.join("key-fill-identifier"));
        }
    }

    let test_dir = candidates
        .iter()
        .find(|p| p.join("index.html").exists())
        .cloned()
        .ok_or_else(|| {
            format!(
                "key-fill-identifier assets not found. Tried: {}",
                candidates
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect::<Vec<_>>()
                    .join("; ")
            )
        })?;

    http_server::start_server(state.test_server.clone(), port, test_dir).await
}

/// Stop the test HTTP server
#[tauri::command]
async fn stop_test_server(state: tauri::State<'_, AppState>) -> Result<(), String> {
    http_server::stop_server(state.test_server.clone()).await
}

/// Get the URL of the test server (if running)
#[tauri::command]
async fn get_test_server_url(state: tauri::State<'_, AppState>) -> Result<Option<String>, String> {
    Ok(http_server::get_server_url(state.test_server.clone()).await)
}

/// Start a channel test by loading fill/key identifier patterns
#[tauri::command]
async fn test_channel(
    channel: u32,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    // Get the test server URL
    let server_url = http_server::get_server_url(state.test_server.clone())
        .await
        .ok_or_else(|| "Test server is not running. Start it first.".to_string())?;

    // Load the test pattern
    let client = state.amcp_client.lock().await;
    client
        .start_channel_test(channel, &server_url)
        .await
        .map_err(|e| e.to_string())
}

/// Stop a channel test by clearing the test layers
#[tauri::command]
async fn stop_channel_test(
    channel: u32,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let client = state.amcp_client.lock().await;
    client
        .stop_channel_test(channel)
        .await
        .map_err(|e| e.to_string())
}

/// Test all configured channels
#[tauri::command]
async fn test_all_channels(
    channel_count: u32,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    // Get the test server URL
    let server_url = http_server::get_server_url(state.test_server.clone())
        .await
        .ok_or_else(|| "Test server is not running. Start it first.".to_string())?;

    // Load test patterns on all channels
    let client = state.amcp_client.lock().await;
    for channel in 1..=channel_count {
        client
            .start_channel_test(channel, &server_url)
            .await
            .map_err(|e| format!("Failed to test channel {}: {}", channel, e))?;
    }

    Ok(())
}

/// Stop all channel tests
#[tauri::command]
async fn stop_all_channel_tests(
    channel_count: u32,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let client = state.amcp_client.lock().await;
    client
        .stop_all_channel_tests(channel_count)
        .await
        .map_err(|e| e.to_string())
}

// ============================================================================
// TSL UMD Tally Monitor Commands
// ============================================================================

/// Start the TSL 3.1 UMD listener on the given UDP port (default 8900)
#[tauri::command]
async fn start_tsl_monitor(
    port: Option<u16>,
    state: tauri::State<'_, AppState>,
) -> Result<u16, String> {
    tsl::start_monitor(state.tsl_monitor.clone(), port).await
}

/// Stop the TSL UMD listener
#[tauri::command]
async fn stop_tsl_monitor(state: tauri::State<'_, AppState>) -> Result<(), String> {
    tsl::stop_monitor(state.tsl_monitor.clone()).await
}

/// Get the current TSL UMD displays (latest message per address)
#[tauri::command]
async fn get_tsl_displays(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<tsl::TslDisplay>, String> {
    Ok(tsl::snapshot(state.tsl_monitor.clone()).await)
}

/// Get the TSL UMD listener port, or null if it is not running
#[tauri::command]
async fn tsl_monitor_port(state: tauri::State<'_, AppState>) -> Result<Option<u16>, String> {
    Ok(tsl::monitor_port(state.tsl_monitor.clone()).await)
}

// ============================================================================
// CasparCG Server Process Commands
// ============================================================================

/// Terminate a process and its whole child tree (CEF/scanner subprocesses).
fn kill_process_tree(pid: u32) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let _ = std::process::Command::new("taskkill")
            .args(["/F", "/T", "/PID", &pid.to_string()])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
    }
    #[cfg(not(windows))]
    {
        let _ = std::process::Command::new("kill").arg(pid.to_string()).output();
    }
}

/// Terminate any stray casparcg.exe a previous session left running, so it cannot
/// keep holding the DeckLink card or the AMCP port.
fn kill_stale_casparcg() {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let _ = std::process::Command::new("taskkill")
            .args(["/F", "/T", "/IM", "casparcg.exe"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
        let _ = std::process::Command::new("taskkill")
            .args(["/F", "/T", "/IM", "scanner.exe"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
    }
}

/// Terminate only casparcg.exe (and its CEF children), leaving scanner.exe alone.
/// Used before a supervised restart: the crashed server's orphaned CEF helpers
/// can keep holding the DeckLink card, which would make the fresh instance fail
/// "Could not enable primary video output" — but the media scanner is healthy
/// and must survive the server bouncing under it.
fn kill_casparcg_only() {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let _ = std::process::Command::new("taskkill")
            .args(["/F", "/T", "/IM", "casparcg.exe"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
    }
}

// --- Supervisor policy ---------------------------------------------------------
// The launcher's second job (after config) is to keep the server and scanner
// alive deterministically. These constants encode that policy in one place.

/// How often the supervisor checks the server and scanner.
const SUPERVISOR_TICK: Duration = Duration::from_secs(3);
/// Pause before relaunching a crashed server, so a hard-failing config cannot
/// spin the CPU between attempts.
const SERVER_RESTART_BACKOFF: Duration = Duration::from_secs(2);
/// CasparCG's documented "please restart me" exit code (see the stock
/// casparcg_auto_restart.bat, which restarts on ERRORLEVEL >= 5). Codes below
/// this are a clean or fatal shutdown we must not fight.
const CASPAR_RESTART_EXIT_CODE: i32 = 5;
/// Crash-loop guard: at most this many crash-restarts within `CRASH_WINDOW`
/// before the supervisor gives up. Without it an unrenderable config (e.g. the
/// AMD GPU mixer that black-screens and crashes) would thrash the machine.
const MAX_SERVER_CRASHES: usize = 3;
/// Rolling window over which `MAX_SERVER_CRASHES` is counted.
const CRASH_WINDOW: Duration = Duration::from_secs(60);

/// Spawn casparcg.exe from `dir`, streaming its console output into the GUI as
/// `caspar-log` events (no separate console window) — the embedded live log the
/// classic launcher is built around. The config file is expected to already be
/// written. Returns the child so the caller can track and supervise it.
fn spawn_caspar(dir: &std::path::Path, app: &tauri::AppHandle) -> Result<std::process::Child, String> {
    let exe = dir.join("casparcg.exe");
    let mut command = std::process::Command::new(&exe);
    command
        .current_dir(dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    let mut child = command
        .spawn()
        .map_err(|e| format!("Failed to launch CasparCG: {}", e))?;

    // Blocking reads run on their own threads so the async runtime is never
    // stalled; they end at EOF when the process exits.
    for stream in [
        child.stdout.take().map(StdStream::Out),
        child.stderr.take().map(StdStream::Err),
    ]
    .into_iter()
    .flatten()
    {
        let app = app.clone();
        std::thread::spawn(move || {
            use std::io::BufRead;
            let lines = match stream {
                StdStream::Out(out) => Box::new(std::io::BufReader::new(out).lines())
                    as Box<dyn Iterator<Item = std::io::Result<String>>>,
                StdStream::Err(err) => Box::new(std::io::BufReader::new(err).lines()),
            };
            for line in lines.map_while(Result::ok) {
                let _ = app.emit("caspar-log", line);
            }
        });
    }

    let _ = app.emit("caspar-log", format!("[launcher] started {}", exe.display()));
    Ok(child)
}

/// Return to a clean stopped state from inside the supervisor: stop wanting the
/// server, kill the media scanner, and tell the GUI its endpoint is gone. Used
/// when the server exits cleanly/fatally or the crash-loop guard trips, so the
/// machine is not left with an orphaned scanner and a stale endpoint readout.
async fn supervisor_stand_down(
    should_run: &Arc<AtomicBool>,
    scanner_arc: &Arc<Mutex<Option<std::process::Child>>>,
    app: &tauri::AppHandle,
) {
    should_run.store(false, Ordering::Release);
    if let Some(mut scanner) = scanner_arc.lock().await.take() {
        kill_process_tree(scanner.id());
        let _ = scanner.wait();
    }
    let _ = app.emit("scanner-endpoint", serde_json::Value::Null);
}

/// Launch the CasparCG media scanner (scanner.exe) from the install directory,
/// streaming its output into the GUI log prefixed with [scanner]. CasparCG 2.x
/// queries the scanner over HTTP for CLS/TLS/THUMBNAIL listings and thumbnails,
/// so a client cannot browse media without it. Returns the child if it launched.
///
/// `host`/`port` pin the scanner's HTTP listener. The media-scanner reads nconf
/// keys with a "__" separator, so `http__host`/`http__port` populate its
/// `{ http: { host, port } }` config. CasparCG must query this exact endpoint —
/// `start_caspar_server` writes the matching `<amcp><media-server>` block.
fn spawn_scanner(
    dir: &std::path::Path,
    app: &tauri::AppHandle,
    host: &str,
    port: u16,
) -> Option<std::process::Child> {
    let exe = dir.join("scanner.exe");
    if !exe.exists() {
        return None;
    }
    let mut command = std::process::Command::new(&exe);
    command
        .current_dir(dir)
        .env("http__host", host)
        .env("http__port", port.to_string())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    let mut child = command.spawn().ok()?;
    for stream in [
        child.stdout.take().map(StdStream::Out),
        child.stderr.take().map(StdStream::Err),
    ]
    .into_iter()
    .flatten()
    {
        let app = app.clone();
        std::thread::spawn(move || {
            use std::io::BufRead;
            let lines = match stream {
                StdStream::Out(out) => Box::new(std::io::BufReader::new(out).lines())
                    as Box<dyn Iterator<Item = std::io::Result<String>>>,
                StdStream::Err(err) => Box::new(std::io::BufReader::new(err).lines()),
            };
            for line in lines.map_while(Result::ok) {
                let _ = app.emit("caspar-log", format!("[scanner] {}", line));
            }
        });
    }
    Some(child)
}

/// Write the active configuration to casparcg.config and launch casparcg.exe
/// from the configured installation directory.
#[tauri::command]
async fn start_caspar_server(
    mut config: GlobalConfig,
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    // Already running? (Reap a process that has since exited.)
    {
        let mut proc = state.caspar_process.lock().await;
        match proc.as_mut().map(|c| c.try_wait()) {
            Some(Ok(None)) => return Err("CasparCG server is already running".to_string()),
            Some(_) => *proc = None,
            None => {}
        }
    }

    // Clean slate: terminate any stray casparcg.exe a previous session left
    // behind. Otherwise it keeps the DeckLink card and AMCP port, and the new
    // instance fails with "Could not enable primary video output".
    kill_stale_casparcg();

    // Resolve the installation directory and executable.
    let caspar_path = {
        let settings = state.gui_settings.lock().await;
        settings
            .caspar_path
            .clone()
            .ok_or_else(|| "CasparCG path is not set — complete setup first".to_string())?
    };
    let dir = PathBuf::from(&caspar_path);
    let exe = dir.join("casparcg.exe");
    if !exe.exists() {
        return Err(format!("casparcg.exe not found in {}", dir.display()));
    }

    // Pin the media scanner to a free loopback port and point CasparCG at it.
    // CasparCG proxies CLS/TLS/THUMBNAIL to the scanner over HTTP; the stock port
    // 8000 routinely clashes with another local web service on a shared box, and
    // then the scanner cannot bind it and every listing fails with "Invalid
    // Response". Resolve the port now so the written config and the spawned
    // scanner always agree on the same endpoint.
    let scanner_host = system::scanner::HOST.to_string();
    let scanner_port = system::scanner::pick_port();
    config.caspar.amcp.media_server = Some(MediaServer {
        host: scanner_host.clone(),
        port: scanner_port,
    });

    // Write the active configuration so the server starts with what is shown.
    let xml = generate_caspar_xml(&config.caspar)
        .map_err(|e| format!("Failed to generate config: {}", e))?;
    std::fs::write(dir.join("casparcg.config"), xml)
        .map_err(|e| format!("Failed to write casparcg.config: {}", e))?;

    // Mark the server as wanted before launching so the supervisor (below) keeps
    // it alive; Stop clears this, so a deliberate stop is never read as a crash.
    state.server_should_run.store(true, Ordering::Release);

    // Launch casparcg.exe with its console streamed into the embedded GUI log.
    let child = match spawn_caspar(&dir, &app) {
        Ok(child) => child,
        Err(e) => {
            state.server_should_run.store(false, Ordering::Release);
            return Err(e);
        }
    };
    *state.caspar_process.lock().await = Some(child);

    // Launch the media scanner alongside the server so a connected client can
    // list media/templates and fetch thumbnails (CLS/TLS/THUMBNAIL). Surface the
    // resolved endpoint both as a log line and as a structured `scanner-endpoint`
    // event so the GUI can show which port it landed on — on a busy box this is
    // not the stock 8000, and that fact is needed to make sense of the listings.
    if let Some(scanner) = spawn_scanner(&dir, &app, &scanner_host, scanner_port) {
        let on_preferred = scanner_port == system::scanner::PREFERRED_PORT;
        let msg = if on_preferred {
            format!("[launcher] media scanner on {scanner_host}:{scanner_port}")
        } else {
            format!(
                "[launcher] preferred port {} busy — media scanner on {scanner_host}:{scanner_port} instead",
                system::scanner::PREFERRED_PORT
            )
        };
        let _ = app.emit("caspar-log", msg);
        let _ = app.emit(
            "scanner-endpoint",
            serde_json::json!({
                "host": scanner_host,
                "port": scanner_port,
                "isDefault": on_preferred,
            }),
        );
        *state.scanner_process.lock().await = Some(scanner);
    } else {
        let _ = app.emit(
            "caspar-log",
            "[launcher] scanner.exe not found — media listing will be unavailable".to_string(),
        );
    }

    // Supervisor: keep both the server and the scanner alive while the server is
    // wanted. The scanner is safe to relaunch freely. The server is restarted on
    // a crash or its restart-request exit code, but a crash-loop guard stops it
    // thrashing on a config the machine cannot render, and a clean/fatal exit is
    // left to stand. A user Stop clears `server_should_run`, so a deliberate stop
    // is never mistaken for a crash.
    {
        let scanner_arc = state.scanner_process.clone();
        let caspar_arc = state.caspar_process.clone();
        let should_run = state.server_should_run.clone();
        let app2 = app.clone();
        let dir2 = dir.clone();
        let scanner_host2 = scanner_host.clone();
        tauri::async_runtime::spawn(async move {
            let mut crashes: Vec<Instant> = Vec::new();
            loop {
                tokio::time::sleep(SUPERVISOR_TICK).await;
                if !should_run.load(Ordering::Acquire) {
                    break; // deliberate Stop / app exit — teardown is the caller's job
                }

                // --- Server: detect an exit and decide whether to restart. ---
                let exit_code: Option<Option<i32>> = {
                    let mut cp = caspar_arc.lock().await;
                    match cp.as_mut().map(|c| c.try_wait()) {
                        Some(Ok(Some(status))) => {
                            *cp = None; // reaped — we own the restart from here
                            Some(status.code())
                        }
                        Some(Ok(None)) => None, // still running
                        Some(Err(_)) => {
                            *cp = None;
                            Some(None) // wait failed — treat as gone, code unknown
                        }
                        None => Some(None), // no handle — gone
                    }
                };

                if let Some(code) = exit_code {
                    // A Stop may have raced in while we looked — re-check intent.
                    if !should_run.load(Ordering::Acquire) {
                        break;
                    }

                    // Codes below 5 are a clean or fatal shutdown (e.g. a config
                    // CasparCG refuses); restarting would only fail the same way.
                    let restart = !matches!(code, Some(c) if c < CASPAR_RESTART_EXIT_CODE);
                    if !restart {
                        let shown = code.map(|c| c.to_string()).unwrap_or_else(|| "?".into());
                        let _ = app2.emit(
                            "caspar-log",
                            format!("[launcher] CasparCG exited (code {shown}) — not restarting"),
                        );
                        supervisor_stand_down(&should_run, &scanner_arc, &app2).await;
                        break;
                    }

                    // Code 5 is a deliberate restart request, not a crash; only
                    // genuine crashes count against the loop guard.
                    let is_crash = !matches!(code, Some(CASPAR_RESTART_EXIT_CODE));
                    if is_crash {
                        let now = Instant::now();
                        crashes.retain(|t| now.duration_since(*t) < CRASH_WINDOW);
                        crashes.push(now);
                        if crashes.len() > MAX_SERVER_CRASHES {
                            let _ = app2.emit(
                                "caspar-log",
                                format!(
                                    "[launcher] CasparCG crashed {} times in {}s — giving up. Check the GPU/config, then press Start.",
                                    crashes.len(),
                                    CRASH_WINDOW.as_secs()
                                ),
                            );
                            supervisor_stand_down(&should_run, &scanner_arc, &app2).await;
                            break;
                        }
                    }

                    let reason = match code {
                        Some(CASPAR_RESTART_EXIT_CODE) => "requested a restart".to_string(),
                        Some(c) => format!("crashed (code {c})"),
                        None => "stopped unexpectedly".to_string(),
                    };
                    let _ = app2.emit("caspar-log", format!("[launcher] CasparCG {reason} — restarting…"));

                    tokio::time::sleep(SERVER_RESTART_BACKOFF).await;
                    if !should_run.load(Ordering::Acquire) {
                        break;
                    }
                    // Clear orphaned CEF children that may still hold the card —
                    // but not the healthy scanner running under it.
                    kill_casparcg_only();
                    match spawn_caspar(&dir2, &app2) {
                        Ok(mut child) => {
                            // A Stop may have raced in during launch — if the
                            // server is no longer wanted, do not leave an orphan
                            // casparcg.exe holding the card.
                            if !should_run.load(Ordering::Acquire) {
                                kill_process_tree(child.id());
                                let _ = child.wait();
                                break;
                            }
                            *caspar_arc.lock().await = Some(child);
                        }
                        Err(e) => {
                            let _ = app2.emit("caspar-log", format!("[launcher] restart failed: {e}"));
                            supervisor_stand_down(&should_run, &scanner_arc, &app2).await;
                            break;
                        }
                    }
                    continue; // the scanner is checked on the next tick
                }

                // --- Scanner: relaunch on the same port if it has died. ---
                let mut sp = scanner_arc.lock().await;
                let scanner_dead = !matches!(sp.as_mut().map(|c| c.try_wait()), Some(Ok(None)));
                if scanner_dead {
                    if let Some(child) = spawn_scanner(&dir2, &app2, &scanner_host2, scanner_port) {
                        let _ = app2.emit(
                            "caspar-log",
                            format!(
                                "[launcher] media scanner restarted on {scanner_host2}:{scanner_port}"
                            ),
                        );
                        *sp = Some(child);
                    }
                }
            }
        });
    }

    Ok(())
}

/// Helper to thread either child stream through one spawn loop.
enum StdStream {
    Out(std::process::ChildStdout),
    Err(std::process::ChildStderr),
}

/// Stop the launched CasparCG server process.
#[tauri::command]
async fn stop_caspar_server(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    // Clear the desired-state flag first so the supervisor sees a deliberate stop
    // and never restarts the server we are about to kill.
    state.server_should_run.store(false, Ordering::Release);

    // Take the server handle so the supervisor sees it gone too.
    let server = state.caspar_process.lock().await.take();

    // Stop the media scanner alongside the server, and tell the GUI its endpoint
    // is gone so the panel does not imply a live scanner.
    if let Some(mut scanner) = state.scanner_process.lock().await.take() {
        kill_process_tree(scanner.id());
        let _ = scanner.wait();
    }
    let _ = app.emit("scanner-endpoint", serde_json::Value::Null);

    if let Some(mut child) = server {
        // Kill the whole tree, not just the direct child — CasparCG spawns CEF
        // subprocesses that would otherwise survive and keep holding the card.
        kill_process_tree(child.id());
        let _ = child.wait();
        Ok(())
    } else {
        Err("CasparCG server is not running".to_string())
    }
}

/// Whether the launched CasparCG server process is still running.
#[tauri::command]
async fn caspar_server_running(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    let mut proc = state.caspar_process.lock().await;
    match proc.as_mut().map(|c| c.try_wait()) {
        Some(Ok(None)) => Ok(true),
        Some(_) => {
            *proc = None;
            Ok(false)
        }
        None => Ok(false),
    }
}

// ============================================================================
// System Info Commands
// ============================================================================

/// Get NDI Tools version
#[tauri::command]
async fn get_ndi_version() -> Result<Option<String>, String> {
    Ok(system::ndi::get_ndi_version())
}

/// Get Media Scanner version
#[tauri::command]
async fn get_scanner_version(url: Option<String>) -> Result<Option<String>, String> {
    Ok(system::scanner::get_scanner_version(url.as_deref()).await)
}

/// Get this host's primary IPv4 — the address a remote operator's client connects
/// to. Returns None when only loopback/overlay addresses are available.
#[tauri::command]
async fn get_primary_ip() -> Result<Option<String>, String> {
    Ok(system::network::primary_ip())
}

/// Get all system version information
#[tauri::command]
async fn get_system_versions(state: tauri::State<'_, AppState>) -> Result<system::SystemVersions, String> {
    let mut versions = system::collect_system_info().await;

    // Try to get CasparCG version if connected
    let client = state.amcp_client.lock().await;
    if client.is_connected() {
        if let Ok(version) = client.version().await {
            versions.caspar_version = Some(version);
        }
    }

    Ok(versions)
}

// ============================================================================
// GUI Settings Commands
// ============================================================================

/// Get GUI settings
#[tauri::command]
async fn get_gui_settings(state: tauri::State<'_, AppState>) -> Result<GuiSettings, String> {
    let settings = state.gui_settings.lock().await;
    Ok(settings.clone())
}

/// Save GUI settings
#[tauri::command]
async fn save_gui_settings(
    settings: GuiSettings,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut current = state.gui_settings.lock().await;
    *current = settings;
    current.save().map_err(|e| format!("Failed to save settings: {}", e))
}

/// Set CasparCG installation path
#[tauri::command]
async fn set_caspar_path(
    path: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut settings = state.gui_settings.lock().await;
    settings.caspar_path = Some(path.clone());
    settings.save().map_err(|e| format!("Failed to save settings: {}", e))?;

    // Create profiles directory if it doesn't exist
    let profiles_dir = PathBuf::from(&path).join("caspar-gui-profiles");
    if !profiles_dir.exists() {
        std::fs::create_dir_all(&profiles_dir)
            .map_err(|e| format!("Failed to create profiles directory: {}", e))?;
    }

    Ok(())
}

// ============================================================================
// File Dialog Commands
// ============================================================================

/// Pick a folder using native dialog
#[tauri::command]
async fn pick_folder(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    let result = app
        .dialog()
        .file()
        .blocking_pick_folder();

    Ok(result.map(|p| p.to_string()))
}

/// Pick a config file using native dialog
#[tauri::command]
async fn pick_config_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    let result = app
        .dialog()
        .file()
        .add_filter("CasparCG Config", &["config"])
        .add_filter("JSON Profile", &["json"])
        .add_filter("All Files", &["*"])
        .blocking_pick_file();

    Ok(result.map(|p| p.to_string()))
}

/// Pick location to save config file
#[tauri::command]
async fn pick_save_location(
    default_name: String,
    app: tauri::AppHandle,
) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    let result = app
        .dialog()
        .file()
        .set_file_name(&default_name)
        .add_filter("JSON Profile", &["json"])
        .add_filter("CasparCG Config", &["config"])
        .blocking_save_file();

    Ok(result.map(|p| p.to_string()))
}

// ============================================================================
// Tauri Plugin Registration
// ============================================================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            // Config commands
            load_caspar_config,
            save_caspar_config,
            load_global_config,
            save_global_config,
            export_to_caspar_xml,
            create_default_config,
            list_profiles,
            // DeckLink commands
            list_decklink_devices,
            get_decklink_info,
            set_decklink_label,
            set_decklink_duplex_mode,
            get_decklink_driver_version,
            get_decklink_status,
            start_decklink_output_test,
            stop_decklink_output_test,
            // AMCP commands
            amcp_connect,
            amcp_disconnect,
            amcp_is_connected,
            amcp_connection_info,
            amcp_version,
            amcp_info_system,
            amcp_send_command,
            // Test server commands
            start_test_server,
            stop_test_server,
            get_test_server_url,
            test_channel,
            stop_channel_test,
            test_all_channels,
            stop_all_channel_tests,
            // TSL UMD monitor commands
            start_tsl_monitor,
            stop_tsl_monitor,
            get_tsl_displays,
            tsl_monitor_port,
            // CasparCG server process commands
            start_caspar_server,
            stop_caspar_server,
            caspar_server_running,
            // System info commands
            get_ndi_version,
            get_scanner_version,
            get_primary_ip,
            get_system_versions,
            // GUI settings commands
            get_gui_settings,
            save_gui_settings,
            set_caspar_path,
            // File dialog commands
            pick_folder,
            pick_config_file,
            pick_save_location,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            // When the GUI exits, kill the launched server and its tree so no
            // casparcg.exe is left holding the DeckLink card.
            if let tauri::RunEvent::Exit = event {
                // Stop any direct SDI output tests (releases the cards' outputs).
                decklink::output_test_stop_all();

                let app_state = app_handle.state::<AppState>();
                // Stop wanting the server so the supervisor cannot race a restart
                // against shutdown.
                app_state.server_should_run.store(false, Ordering::Release);
                let pid = app_state
                    .caspar_process
                    .try_lock()
                    .ok()
                    .and_then(|mut guard| guard.take())
                    .map(|child| child.id());
                if let Some(pid) = pid {
                    kill_process_tree(pid);
                }
                // Also stop the media scanner.
                let scanner_pid = app_state
                    .scanner_process
                    .try_lock()
                    .ok()
                    .and_then(|mut guard| guard.take())
                    .map(|child| child.id());
                if let Some(pid) = scanner_pid {
                    kill_process_tree(pid);
                }
            }
        });
}
