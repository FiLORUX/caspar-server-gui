// CasparCG Server GUI - Rust Backend
// Provides configuration management, AMCP communication, and DeckLink integration

mod amcp;
mod config;
mod decklink;
mod http_server;
mod system;
mod tsl;

use std::path::PathBuf;
use std::sync::Arc;
use tauri::{Emitter, Manager};
use tokio::sync::Mutex;

use config::{
    generate_caspar_xml, parse_caspar_xml, CasparConfig, GlobalConfig, GuiSettings,
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

/// Launch the CasparCG media scanner (scanner.exe) from the install directory,
/// streaming its output into the GUI log prefixed with [scanner]. CasparCG 2.x
/// queries the scanner over HTTP for CLS/TLS/THUMBNAIL listings and thumbnails,
/// so a client cannot browse media without it. Returns the child if it launched.
fn spawn_scanner(dir: &std::path::Path, app: &tauri::AppHandle) -> Option<std::process::Child> {
    let exe = dir.join("scanner.exe");
    if !exe.exists() {
        return None;
    }
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
    config: GlobalConfig,
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

    // Write the active configuration so the server starts with what is shown.
    let xml = generate_caspar_xml(&config.caspar)
        .map_err(|e| format!("Failed to generate config: {}", e))?;
    std::fs::write(dir.join("casparcg.config"), xml)
        .map_err(|e| format!("Failed to write casparcg.config: {}", e))?;

    // Capture the server's console output and stream it into the GUI as
    // `caspar-log` events instead of opening a separate console window — this is
    // the embedded live log the classic launcher (CasparLauncher) is built around.
    let mut command = std::process::Command::new(&exe);
    command
        .current_dir(&dir)
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

    // Forward each output line to the front end. Blocking reads run on their own
    // threads so the async runtime is never stalled; they end at EOF when the
    // process exits.
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
            match stream {
                StdStream::Out(out) => {
                    for line in std::io::BufReader::new(out).lines().map_while(Result::ok) {
                        let _ = app.emit("caspar-log", line);
                    }
                }
                StdStream::Err(err) => {
                    for line in std::io::BufReader::new(err).lines().map_while(Result::ok) {
                        let _ = app.emit("caspar-log", line);
                    }
                }
            }
        });
    }

    let _ = app.emit("caspar-log", format!("[launcher] started {}", exe.display()));
    *state.caspar_process.lock().await = Some(child);

    // Launch the media scanner alongside the server so a connected client can
    // list media/templates and fetch thumbnails (CLS/TLS/THUMBNAIL).
    if let Some(scanner) = spawn_scanner(&dir, &app) {
        let _ = app.emit("caspar-log", "[launcher] started media scanner".to_string());
        *state.scanner_process.lock().await = Some(scanner);
    } else {
        let _ = app.emit(
            "caspar-log",
            "[launcher] scanner.exe not found — media listing will be unavailable".to_string(),
        );
    }

    // Watchdog: while the server runs, relaunch the scanner if it dies (it is
    // the more crash-prone of the two and safe to restart). The server itself is
    // left to the user's Start/Stop — auto-restarting it would fight Stop and
    // thrash on a config the GPU cannot render.
    {
        let scanner_arc = state.scanner_process.clone();
        let caspar_arc = state.caspar_process.clone();
        let app2 = app.clone();
        let dir2 = dir.clone();
        tauri::async_runtime::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                {
                    let mut cp = caspar_arc.lock().await;
                    let server_alive = matches!(cp.as_mut().map(|c| c.try_wait()), Some(Ok(None)));
                    if !server_alive {
                        break; // server gone — stop watching
                    }
                }
                let mut sp = scanner_arc.lock().await;
                let scanner_dead = !matches!(sp.as_mut().map(|c| c.try_wait()), Some(Ok(None)));
                if scanner_dead {
                    if let Some(child) = spawn_scanner(&dir2, &app2) {
                        let _ = app2.emit("caspar-log", "[launcher] media scanner restarted".to_string());
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
async fn stop_caspar_server(state: tauri::State<'_, AppState>) -> Result<(), String> {
    // Take the server handle first so the scanner watchdog sees the server gone
    // and stops before we kill the scanner (otherwise it could relaunch it).
    let server = state.caspar_process.lock().await.take();

    // Stop the media scanner alongside the server.
    if let Some(mut scanner) = state.scanner_process.lock().await.take() {
        kill_process_tree(scanner.id());
        let _ = scanner.wait();
    }

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
