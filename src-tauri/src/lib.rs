pub mod cache;
pub mod commands;
pub mod config_file;
pub mod env_helper;

use commands::{mirrors::*, mise::*, projects::*, system::*, updater::*};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 1. Fix macOS / Linux GUI PATH inheritance
    env_helper::fix_system_path();

    // 2. Build Tauri application with commands registered
    let builder = tauri::Builder::default();

    // `single-instance` must be the first plugin registered: it forwards the
    // second launch's argv to the running instance and exits.
    #[cfg(desktop)]
    let builder = builder.plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
        use tauri::Manager;
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.unminimize();
            let _ = window.show();
            let _ = window.set_focus();
        }
    }));

    builder
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            // Mise commands
            get_runtimes,
            get_remote_versions,
            set_global_version,
            uninstall_runtime_version,
            install_runtime_version,
            cancel_current_install,
            bootstrap_mise_cli,
            open_terminal_for_runtime,
            // System commands
            get_system_status,
            env_helper::save_mise_path,
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
            check_github_latest_release,
        ])
        .run(tauri::generate_context!())
        .expect("error while running envhub tauri application");
}
