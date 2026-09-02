pub mod env_helper;
pub mod commands;

use commands::{
    mise::*,
    system::*,
    mirrors::*,
    projects::*,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 1. Fix macOS / Linux GUI PATH inheritance
    env_helper::fix_system_path();

    // 2. Build Tauri application with commands registered
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            // Mise commands
            get_runtimes,
            get_remote_versions,
            set_global_version,
            uninstall_runtime_version,
            install_runtime_version,
            bootstrap_mise_cli,

            // System commands
            get_system_status,
            get_system_tools,
            install_system_tool,
            get_health_checks,
            auto_fix_health_check,

            // Mirror commands
            get_mirrors,
            set_mirror,
            ping_mirrors,

            // Project commands
            get_projects,
            scan_and_add_project,
            set_project_tool_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running envhub tauri application");
}
