use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use chrono::Local;

const HOSTS_PATH: &str = r"C:\Windows\System32\drivers\etc\hosts";

#[derive(Debug, Serialize, Deserialize)]
struct Metadata {
    active: String,
    profiles: Vec<String>,
}

// Get the profiles directory path
fn get_profiles_dir() -> Result<PathBuf, String> {
    let app_data = dirs::data_dir().ok_or("Could not find AppData directory")?;
    let profiles_dir = app_data.join("HostsPilot").join("profiles");
    
    if !profiles_dir.exists() {
        fs::create_dir_all(&profiles_dir)
            .map_err(|e| format!("Failed to create profiles directory: {}", e))?;
    }
    
    Ok(profiles_dir)
}

// Get metadata file path
fn get_metadata_path() -> Result<PathBuf, String> {
    let profiles_dir = get_profiles_dir()?;
    Ok(profiles_dir.join("metadata.json"))
}

// Read metadata
fn read_metadata() -> Result<Metadata, String> {
    let metadata_path = get_metadata_path()?;
    
    if !metadata_path.exists() {
        // Create default metadata
        let default_metadata = Metadata {
            active: String::new(),
            profiles: Vec::new(),
        };
        save_metadata(&default_metadata)?;
        return Ok(default_metadata);
    }
    
    let content = fs::read_to_string(&metadata_path)
        .map_err(|e| format!("Failed to read metadata: {}", e))?;
    
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse metadata: {}", e))
}

// Save metadata
fn save_metadata(metadata: &Metadata) -> Result<(), String> {
    let metadata_path = get_metadata_path()?;
    let content = serde_json::to_string_pretty(metadata)
        .map_err(|e| format!("Failed to serialize metadata: {}", e))?;
    
    fs::write(&metadata_path, content)
        .map_err(|e| format!("Failed to write metadata: {}", e))
}

#[tauri::command]
fn list_profiles() -> Result<Vec<String>, String> {
    let metadata = read_metadata()?;
    Ok(metadata.profiles)
}

#[tauri::command]
fn get_active_profile() -> Result<String, String> {
    let metadata = read_metadata()?;
    Ok(metadata.active)
}

#[tauri::command]
fn create_profile(name: String) -> Result<(), String> {
    if name.is_empty() {
        return Err("Profile name cannot be empty".to_string());
    }
    
    let mut metadata = read_metadata()?;
    
    if metadata.profiles.contains(&name) {
        return Err("Profile already exists".to_string());
    }
    
    let profiles_dir = get_profiles_dir()?;
    let profile_path = profiles_dir.join(format!("{}.hosts", name));
    
    // Create empty profile file
    fs::write(&profile_path, "# New profile\n")
        .map_err(|e| format!("Failed to create profile: {}", e))?;
    
    metadata.profiles.push(name);
    save_metadata(&metadata)?;
    
    Ok(())
}

#[tauri::command]
fn delete_profile(name: String) -> Result<(), String> {
    let mut metadata = read_metadata()?;
    
    if !metadata.profiles.contains(&name) {
        return Err("Profile does not exist".to_string());
    }
    
    if metadata.active == name {
        return Err("Cannot delete active profile".to_string());
    }
    
    let profiles_dir = get_profiles_dir()?;
    let profile_path = profiles_dir.join(format!("{}.hosts", name));
    
    fs::remove_file(&profile_path)
        .map_err(|e| format!("Failed to delete profile: {}", e))?;
    
    metadata.profiles.retain(|p| p != &name);
    save_metadata(&metadata)?;
    
    Ok(())
}

#[tauri::command]
fn rename_profile(old_name: String, new_name: String) -> Result<(), String> {
    if new_name.is_empty() {
        return Err("Profile name cannot be empty".to_string());
    }
    
    let mut metadata = read_metadata()?;
    
    if !metadata.profiles.contains(&old_name) {
        return Err("Profile does not exist".to_string());
    }
    
    if metadata.profiles.contains(&new_name) {
        return Err("A profile with that name already exists".to_string());
    }
    
    let profiles_dir = get_profiles_dir()?;
    let old_path = profiles_dir.join(format!("{}.hosts", old_name));
    let new_path = profiles_dir.join(format!("{}.hosts", new_name));
    
    fs::rename(&old_path, &new_path)
        .map_err(|e| format!("Failed to rename profile: {}", e))?;
    
    // Update metadata
    if let Some(pos) = metadata.profiles.iter().position(|p| p == &old_name) {
        metadata.profiles[pos] = new_name.clone();
    }
    
    if metadata.active == old_name {
        metadata.active = new_name;
    }
    
    save_metadata(&metadata)?;
    
    Ok(())
}

#[tauri::command]
fn read_profile(name: String) -> Result<String, String> {
    let profiles_dir = get_profiles_dir()?;
    let profile_path = profiles_dir.join(format!("{}.hosts", name));
    
    fs::read_to_string(&profile_path)
        .map_err(|e| format!("Failed to read profile: {}", e))
}

#[tauri::command]
fn write_profile(name: String, content: String) -> Result<(), String> {
    let profiles_dir = get_profiles_dir()?;
    let profile_path = profiles_dir.join(format!("{}.hosts", name));
    
    fs::write(&profile_path, content)
        .map_err(|e| format!("Failed to write profile: {}", e))
}

#[tauri::command]
fn activate_profile(name: String) -> Result<(), String> {
    let metadata = read_metadata()?;
    
    if !metadata.profiles.contains(&name) {
        return Err("Profile does not exist".to_string());
    }
    
    // Backup current hosts file
    backup_hosts()?;
    
    // Read profile content
    let content = read_profile(name.clone())?;
    
    // Write to system hosts file (requires admin)
    fs::write(HOSTS_PATH, content)
        .map_err(|e| format!("Failed to write hosts file (admin rights required): {}", e))?;
    
    // Update active profile in metadata
    let mut metadata = read_metadata()?;
    metadata.active = name;
    save_metadata(&metadata)?;
    
    // Flush DNS
    flush_dns()?;
    
    Ok(())
}

#[tauri::command]
fn read_system_hosts() -> Result<String, String> {
    fs::read_to_string(HOSTS_PATH)
        .map_err(|e| format!("Failed to read system hosts file: {}", e))
}

#[tauri::command]
fn backup_hosts() -> Result<String, String> {
    let profiles_dir = get_profiles_dir()?;
    let backups_dir = profiles_dir.parent().unwrap().join("backups");
    
    if !backups_dir.exists() {
        fs::create_dir_all(&backups_dir)
            .map_err(|e| format!("Failed to create backups directory: {}", e))?;
    }
    
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let backup_path = backups_dir.join(format!("hosts_backup_{}.txt", timestamp));
    
    let content = fs::read_to_string(HOSTS_PATH)
        .map_err(|e| format!("Failed to read hosts file: {}", e))?;
    
    fs::write(&backup_path, content)
        .map_err(|e| format!("Failed to write backup: {}", e))?;
    
    // Clean up old backups, keep only the 25 most recent
    cleanup_old_backups(&backups_dir, 25)?;
    
    Ok(backup_path.to_string_lossy().to_string())
}

fn cleanup_old_backups(backups_dir: &std::path::Path, max_backups: usize) -> Result<(), String> {
    let mut backups: Vec<_> = fs::read_dir(backups_dir)
        .map_err(|e| format!("Failed to read backups directory: {}", e))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().is_file() && 
            entry.path().extension().and_then(|s| s.to_str()) == Some("txt")
        })
        .collect();
    
    // Sort by modification time (oldest first)
    backups.sort_by_key(|entry| {
        entry.metadata()
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
    });
    
    // Delete oldest backups if we exceed the limit
    if backups.len() > max_backups {
        let to_delete = backups.len() - max_backups;
        for entry in backups.iter().take(to_delete) {
            fs::remove_file(entry.path())
                .map_err(|e| format!("Failed to delete old backup: {}", e))?;
        }
    }
    
    Ok(())
}

#[tauri::command]
fn list_backups() -> Result<Vec<String>, String> {
    let profiles_dir = get_profiles_dir()?;
    let backups_dir = profiles_dir.parent().unwrap().join("backups");
    
    if !backups_dir.exists() {
        return Ok(Vec::new());
    }
    
    let entries = fs::read_dir(&backups_dir)
        .map_err(|e| format!("Failed to read backups directory: {}", e))?;
    
    let mut backups = Vec::new();
    for entry in entries {
        if let Ok(entry) = entry {
            if let Some(name) = entry.file_name().to_str() {
                backups.push(name.to_string());
            }
        }
    }
    
    backups.sort();
    backups.reverse(); // Most recent first
    
    Ok(backups)
}

#[tauri::command]
fn read_backup(backup_name: String) -> Result<String, String> {
    let profiles_dir = get_profiles_dir()?;
    let backups_dir = profiles_dir.parent().unwrap().join("backups");
    let backup_path = backups_dir.join(&backup_name);
    
    if !backup_path.exists() {
        return Err("Backup file does not exist".to_string());
    }
    
    fs::read_to_string(&backup_path)
        .map_err(|e| format!("Failed to read backup: {}", e))
}

#[tauri::command]
fn restore_hosts(backup_name: String) -> Result<(), String> {
    let profiles_dir = get_profiles_dir()?;
    let backups_dir = profiles_dir.parent().unwrap().join("backups");
    let backup_path = backups_dir.join(&backup_name);
    
    if !backup_path.exists() {
        return Err("Backup file does not exist".to_string());
    }
    
    // Backup current hosts before restoring
    backup_hosts()?;
    
    let content = fs::read_to_string(&backup_path)
        .map_err(|e| format!("Failed to read backup: {}", e))?;
    
    fs::write(HOSTS_PATH, content)
        .map_err(|e| format!("Failed to restore hosts file (admin rights required): {}", e))?;
    
    flush_dns()?;
    
    Ok(())
}

#[tauri::command]
fn delete_backup(backup_name: String) -> Result<(), String> {
    let profiles_dir = get_profiles_dir()?;
    let backups_dir = profiles_dir.parent().unwrap().join("backups");
    let backup_path = backups_dir.join(&backup_name);
    
    if !backup_path.exists() {
        return Err("Backup file does not exist".to_string());
    }
    
    fs::remove_file(&backup_path)
        .map_err(|e| format!("Failed to delete backup: {}", e))?;
    
    Ok(())
}

#[tauri::command]
fn delete_all_backups() -> Result<(), String> {
    let profiles_dir = get_profiles_dir()?;
    let backups_dir = profiles_dir.parent().unwrap().join("backups");
    
    if !backups_dir.exists() {
        return Ok(());
    }
    
    let entries = fs::read_dir(&backups_dir)
        .map_err(|e| format!("Failed to read backups directory: {}", e))?;
    
    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("txt") {
                fs::remove_file(&path)
                    .map_err(|e| format!("Failed to delete backup: {}", e))?;
            }
        }
    }
    
    Ok(())
}

#[tauri::command]
fn flush_dns() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        
        let output = Command::new("ipconfig")
            .arg("/flushdns")
            .output()
            .map_err(|e| format!("Failed to flush DNS: {}", e))?;
        
        if !output.status.success() {
            return Err("DNS flush command failed".to_string());
        }
    }
    
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            list_profiles,
            get_active_profile,
            create_profile,
            delete_profile,
            rename_profile,
            read_profile,
            write_profile,
            activate_profile,
            read_system_hosts,
            backup_hosts,
            list_backups,
            read_backup,
            restore_hosts,
            delete_backup,
            delete_all_backups,
            flush_dns,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
