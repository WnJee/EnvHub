use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;

pub static CONFIG_LOCK: Mutex<()> = Mutex::new(());

pub fn read(path: &Path) -> Result<String, String> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(text),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(e) => Err(format!("Cannot read {}: {e}", path.display())),
    }
}

/// Stage on the same filesystem, preserve permissions and retain a recovery copy.
pub fn write(path: &Path, content: &str) -> Result<(), String> {
    let path = if path.is_symlink() {
        path.canonicalize().map_err(|e| e.to_string())?
    } else {
        path.to_path_buf()
    };
    let parent = path.parent().ok_or("Invalid configuration path")?;
    if path.exists() && !path.is_file() {
        return Err("Configuration target is not a regular file".into());
    }
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let mut staged = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    if path.exists() {
        let metadata = fs::metadata(&path).map_err(|e| e.to_string())?;
        staged
            .as_file()
            .set_permissions(metadata.permissions())
            .map_err(|e| e.to_string())?;
        let backup = tempfile::Builder::new()
            .prefix(".envhub-backup-")
            .tempfile_in(parent)
            .map_err(|e| e.to_string())?;
        fs::copy(&path, backup.path()).map_err(|e| e.to_string())?;
        backup.keep().map_err(|e| e.to_string())?;
    }
    staged
        .write_all(content.as_bytes())
        .map_err(|e| e.to_string())?;
    staged.as_file().sync_all().map_err(|e| e.to_string())?;
    staged
        .persist(&path)
        .map_err(|e| format!("Cannot replace {}: {e}", path.display()))?;
    Ok(())
}

pub fn edit_toml(
    path: &Path,
    edit: impl FnOnce(&mut toml_edit::DocumentMut) -> Result<(), String>,
) -> Result<(), String> {
    let _guard = CONFIG_LOCK.lock().map_err(|e| e.to_string())?;
    let old = read(path)?;
    let mut doc = old
        .parse::<toml_edit::DocumentMut>()
        .map_err(|e| format!("Invalid {}: {e}", path.display()))?;
    edit(&mut doc)?;
    let new = doc.to_string();
    if new != old {
        write(path, &new)?;
    }
    Ok(())
}

pub fn set_tool(doc: &mut toml_edit::DocumentMut, tool: &str, version: &str) -> Result<(), String> {
    if !doc.contains_key("tools") {
        doc["tools"] = toml_edit::Item::Table(toml_edit::Table::new());
    }
    let tools = doc["tools"]
        .as_table_like_mut()
        .ok_or("[tools] is not a table")?;
    if let Some(item) = tools.get_mut(tool) {
        if let Some(options) = item.as_table_like_mut() {
            options.insert("version", toml_edit::value(version));
            return Ok(());
        }
    }
    tools.insert(tool, toml_edit::value(version));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn quoted_tools_and_other_sections_survive() {
        let mut doc =
            "[tools]\n\"node\" = \"20\"\nnodejs = \"18\"\n[env]\nnode_service = \"keep\"\n"
                .parse()
                .unwrap();
        set_tool(&mut doc, "node", "22").unwrap();
        let parsed = doc.to_string().parse::<toml_edit::DocumentMut>().unwrap();
        assert_eq!(parsed["tools"]["node"].as_str(), Some("22"));
        assert_eq!(parsed["tools"]["nodejs"].as_str(), Some("18"));
        assert_eq!(parsed["env"]["node_service"].as_str(), Some("keep"));
    }
    #[test]
    fn invalid_config_is_not_overwritten() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        fs::write(&path, "[broken").unwrap();
        assert!(edit_toml(&path, |d| set_tool(d, "node", "22")).is_err());
        assert_eq!(read(&path).unwrap(), "[broken");
    }
    #[test]
    fn writes_keep_backup() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config");
        fs::write(&path, "old").unwrap();
        write(&path, "new").unwrap();
        assert_eq!(read(&path).unwrap(), "new");
        assert!(fs::read_dir(dir.path()).unwrap().flatten().any(|e| e
            .file_name()
            .to_string_lossy()
            .starts_with(".envhub-backup-")
            && read(&e.path()).unwrap() == "old"));
    }
}
