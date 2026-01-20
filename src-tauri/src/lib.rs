// CasparCG Server GUI - Rust Backend
// Provides configuration management, AMCP communication, and DeckLink integration

mod amcp;
mod config;
mod decklink;
mod system;

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use config::{
    generate_caspar_xml, parse_caspar_xml, CasparConfig, GlobalConfig, GuiSettings,
};
use decklink::{DeckLinkDevice, DuplexMode};

/// Application state shared across commands
pub struct AppState {
    pub amcp_client: Arc<Mutex<amcp::AmcpClient>>,
    pub gui_settings: Arc<Mutex<GuiSettings>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            amcp_client: Arc::new(Mutex::new(amcp::AmcpClient::new())),
            gui_settings: Arc::new(Mutex::new(GuiSettings::load())),
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

/// Set the display label for a DeckLink device
#[tauri::command]
async fn set_decklink_label(
    _persistent_id: String,
    _label: String,
) -> Result<(), String> {
    // Note: DeckLink SDK doesn't support persistent labels
    // This would be stored in our global config instead
    Ok(())
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
            // AMCP commands
            amcp_connect,
            amcp_disconnect,
            amcp_is_connected,
            amcp_connection_info,
            amcp_version,
            amcp_info_system,
            amcp_send_command,
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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
