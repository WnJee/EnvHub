pub mod env_helper;
pub mod commands;

use commands::{
    mise::*,
    system::*,
    mirrors::*,
    projects::*,
    updater::*,
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
            open_terminal_for_runtime,

            // System commands
            get_system_status,
            get_system_tools,
            test_system_tool,
            install_system_tool,
            get_health_checks,
            auto_fix_health_check,
            save_export_file,

            // Mirror commands
            get_mirrors,
            set_mirror,
            ping_mirrors,

            // Project commands
            get_projects,
            scan_and_add_project,
            remove_project,
            set_project_tool_version,
            open_in_editor,
            open_in_terminal,

            // Updater & Browser commands
            open_url_in_browser,
            download_and_install_update,
            open_path_in_file_manager,
            open_installer_file,
            relaunch_application,
        ])
        .run(tauri::generate_context!())
        .expect("error while running envhub tauri application");
}
