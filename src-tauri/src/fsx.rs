//! Unrestricted absolute-path file helpers.
//!
//! The Projects and Themes folders can live anywhere on disk (portable exe
//! dir or a user-chosen location), which is outside the tauri-plugin-fs
//! AppData scope. These commands run in Rust — unaffected by the frontend
//! filesystem scope — so the UI can read/write theme CSS, list fonts and copy
//! font files regardless of where the data folders were relocated.

use std::fs;
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use serde::Serialize;

#[tauri::command(async)]
pub fn read_text_file_abs(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|e| e.to_string())
}

/// Reads a binary file and returns its base64-encoded contents. Used to load
/// uploaded font files into the frontend as `@font-face` data URLs.
#[tauri::command(async)]
pub fn read_file_base64_abs(path: String) -> Result<String, String> {
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    let bytes = fs::read(&path).map_err(|e| e.to_string())?;
    Ok(STANDARD.encode(bytes))
}

#[tauri::command(async)]
pub fn write_text_file_abs(path: String, contents: String) -> Result<(), String> {
    if let Some(parent) = Path::new(&path).parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(&path, contents).map_err(|e| e.to_string())
}

#[tauri::command(async)]
pub fn path_exists_abs(path: String) -> bool {
    Path::new(&path).exists()
}

#[tauri::command(async)]
pub fn make_dir_abs(path: String) -> Result<(), String> {
    fs::create_dir_all(&path).map_err(|e| e.to_string())
}

/// Lists file names in `dir` whose extension is in `exts` (case-insensitive).
/// Pass an empty `exts` to list every file. Non-existent dir → empty list.
#[tauri::command(async)]
pub fn list_dir_files_abs(dir: String, exts: Vec<String>) -> Result<Vec<String>, String> {
    let path = Path::new(&dir);
    if !path.exists() {
        return Ok(vec![]);
    }
    let lower: Vec<String> = exts.iter().map(|e| e.to_lowercase()).collect();
    let mut out = Vec::new();
    for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if lower.is_empty() {
            out.push(name);
            continue;
        }
        let ext = Path::new(&name)
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        if lower.contains(&ext) {
            out.push(name);
        }
    }
    out.sort();
    Ok(out)
}

const IMAGE_EXTS: &[&str] = &["gif", "webp", "png", "jpg", "jpeg"];

/// Absolute paths of image files (gif/webp/png/jpg/jpeg) in `dir`.
/// Direct children only. Missing dir → empty list.
#[tauri::command(async)]
pub fn list_image_files_abs(dir: String) -> Result<Vec<String>, String> {
    let path = Path::new(&dir);
    if !path.exists() {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            continue;
        }
        let p = entry.path();
        let ext = p
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        if !IMAGE_EXTS.contains(&ext.as_str()) {
            continue;
        }
        out.push(p.to_string_lossy().into_owned());
    }
    out.sort();
    Ok(out)
}

/// Copies `src` to an exact destination path (creates parent dirs).
#[tauri::command(async)]
pub fn copy_file_to_abs(src: String, dst: String) -> Result<String, String> {
    let src_path = PathBuf::from(&src);
    let dst_path = PathBuf::from(&dst);
    if let Some(parent) = dst_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::copy(&src_path, &dst_path).map_err(|e| e.to_string())?;
    Ok(dst_path.to_string_lossy().to_string())
}

/// Copies `src` into `dst_dir`, keeping the original file name.
/// Returns the destination absolute path.
#[tauri::command(async)]
pub fn copy_file_abs(src: String, dst_dir: String) -> Result<String, String> {
    let src_path = PathBuf::from(&src);
    let name = src_path
        .file_name()
        .ok_or_else(|| "source has no file name".to_string())?;
    fs::create_dir_all(&dst_dir).map_err(|e| e.to_string())?;
    let dst = Path::new(&dst_dir).join(name);
    fs::copy(&src_path, &dst).map_err(|e| e.to_string())?;
    Ok(dst.to_string_lossy().to_string())
}

/// Deletes a file if it exists. No error when already absent.
#[tauri::command(async)]
pub fn delete_file_abs(path: String) -> Result<(), String> {
    let p = Path::new(&path);
    if p.exists() {
        fs::remove_file(p).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Deletes a directory recursively if it exists. No error when already absent.
#[tauri::command(async)]
pub fn delete_dir_abs(path: String) -> Result<(), String> {
    let p = Path::new(&path);
    if p.exists() {
        fs::remove_dir_all(p).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImportAudioFile {
    pub path: String,
    pub file_name: String,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImportFolderGroup {
    pub name: String,
    pub path: String,
    pub audio: Vec<ImportAudioFile>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImportFolder {
    pub path: String,
    pub name: String,
    pub root_audio: Vec<ImportAudioFile>,
    pub groups: Vec<ImportFolderGroup>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImportDropScan {
    pub files: Vec<ImportAudioFile>,
    pub folders: Vec<ImportFolder>,
    pub skipped: usize,
}

fn normalize_exts(exts: Vec<String>) -> Vec<String> {
    exts.into_iter()
        .map(|e| e.trim_start_matches('.').to_lowercase())
        .collect()
}

fn file_ext(path: &Path) -> String {
    path.extension()
        .map(|v| v.to_string_lossy().to_lowercase())
        .unwrap_or_default()
}

fn matches_ext(path: &Path, exts: &[String]) -> bool {
    exts.contains(&file_ext(path))
}

fn file_name_string(path: &Path) -> String {
    path.file_name()
        .map(|v| v.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned())
}

fn to_audio(path: &Path) -> ImportAudioFile {
    ImportAudioFile {
        path: path.to_string_lossy().into_owned(),
        file_name: file_name_string(path),
    }
}

fn read_error(dir: &Path, e: std::io::Error) -> String {
    format!("Failed to read folder '{}': {}", dir.to_string_lossy(), e)
}

fn collect_audio_recursive(dir: &Path, exts: &[String]) -> Result<Vec<ImportAudioFile>, String> {
    let mut out = Vec::new();
    let mut queue = VecDeque::new();
    queue.push_back(dir.to_path_buf());
    while let Some(current) = queue.pop_front() {
        let entries = fs::read_dir(&current).map_err(|e| read_error(&current, e))?;
        let mut subdirs = Vec::new();
        for entry_res in entries {
            let entry = entry_res.map_err(|e| read_error(&current, e))?;
            let path = entry.path();
            let ty = entry.file_type().map_err(|e| read_error(&current, e))?;
            if ty.is_file() {
                if matches_ext(&path, exts) {
                    out.push(to_audio(&path));
                }
            } else if ty.is_dir() {
                subdirs.push(path);
            }
        }
        subdirs.sort();
        queue.extend(subdirs);
    }
    out.sort_by(|a, b| a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase()));
    Ok(out)
}

fn scan_folder(dir: &Path, exts: &[String]) -> Result<Option<ImportFolder>, String> {
    let entries = fs::read_dir(dir).map_err(|e| read_error(dir, e))?;
    let mut root_audio = Vec::new();
    let mut subdirs = Vec::new();
    for entry_res in entries {
        let entry = entry_res.map_err(|e| read_error(dir, e))?;
        let path = entry.path();
        let ty = entry.file_type().map_err(|e| read_error(dir, e))?;
        if ty.is_file() {
            if matches_ext(&path, exts) {
                root_audio.push(to_audio(&path));
            }
        } else if ty.is_dir() {
            subdirs.push(path);
        }
    }
    root_audio.sort_by(|a, b| a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase()));
    subdirs.sort();

    let mut groups = Vec::new();
    for sub in subdirs {
        let audio = collect_audio_recursive(&sub, exts)?;
        if audio.is_empty() {
            continue;
        }
        groups.push(ImportFolderGroup {
            name: file_name_string(&sub),
            path: sub.to_string_lossy().into_owned(),
            audio,
        });
    }

    if root_audio.is_empty() && groups.is_empty() {
        return Ok(None);
    }

    Ok(Some(ImportFolder {
        path: dir.to_string_lossy().into_owned(),
        name: file_name_string(dir),
        root_audio,
        groups,
    }))
}

/// Classify dropped/picked paths into loose audio files and folders
/// (root-level audio + one group per immediate subfolder, nested audio flattened).
#[tauri::command(async)]
pub fn inspect_import_drop(
    paths: Vec<String>,
    exts: Vec<String>,
) -> Result<ImportDropScan, String> {
    let exts = normalize_exts(exts);
    let mut files = Vec::new();
    let mut folders = Vec::new();
    let mut skipped = 0usize;

    for raw in paths {
        let path = PathBuf::from(&raw);
        let Ok(meta) = fs::metadata(&path) else {
            skipped += 1;
            continue;
        };
        if meta.is_file() {
            if matches_ext(&path, &exts) {
                files.push(to_audio(&path));
            } else {
                skipped += 1;
            }
        } else if meta.is_dir() {
            match scan_folder(&path, &exts)? {
                Some(folder) => folders.push(folder),
                None => skipped += 1,
            }
        } else {
            skipped += 1;
        }
    }

    Ok(ImportDropScan {
        files,
        folders,
        skipped,
    })
}

#[tauri::command(async)]
pub fn paths_exist_abs(paths: Vec<String>) -> Vec<bool> {
    paths.into_iter().map(|p| Path::new(&p).exists()).collect()
}

#[derive(Serialize)]
pub struct SoundFileMeta {
    pub path: String,
    pub size: u64,
    pub mtime: u64,
}

/// Cheap `stat` for board sort (size + mtime). Missing files are omitted.
#[tauri::command(async)]
pub fn get_sound_file_meta(paths: Vec<String>) -> Vec<SoundFileMeta> {
    let mut out = Vec::with_capacity(paths.len());
    for path in paths {
        let Ok(meta) = fs::metadata(&path) else { continue };
        let size = meta.len();
        let mtime = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        out.push(SoundFileMeta { path, size, mtime });
    }
    out
}

#[derive(Serialize)]
pub struct FileNameMatch {
    pub name: String,
    pub paths: Vec<String>,
}

/// BFS scan of `roots` up to `max_depth` for files whose basename is in `names`.
#[tauri::command(async)]
pub fn find_files_by_names(
    roots: Vec<String>,
    names: Vec<String>,
    max_depth: usize,
) -> Result<Vec<FileNameMatch>, String> {
    use std::collections::HashMap;
    let wanted: std::collections::HashSet<String> = names
        .iter()
        .map(|n| n.to_lowercase())
        .collect();
    let mut found: HashMap<String, Vec<String>> = HashMap::new();

    for root in roots {
        let root_path = PathBuf::from(&root);
        if !root_path.exists() {
            continue;
        }
        let mut queue: VecDeque<(PathBuf, usize)> = VecDeque::new();
        queue.push_back((root_path, 0));
        while let Some((dir, depth)) = queue.pop_front() {
            let entries = match fs::read_dir(&dir) {
                Ok(e) => e,
                Err(_) => continue,
            };
            for entry_res in entries {
                let Ok(entry) = entry_res else { continue };
                let path = entry.path();
                let Ok(ty) = entry.file_type() else { continue };
                if ty.is_file() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if wanted.contains(&name.to_lowercase()) {
                        found.entry(name).or_default().push(path.to_string_lossy().to_string());
                    }
                } else if ty.is_dir() && depth < max_depth {
                    queue.push_back((path, depth + 1));
                }
            }
        }
    }

    Ok(found
        .into_iter()
        .map(|(name, paths)| FileNameMatch { name, paths })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::{inspect_import_drop, list_image_files_abs};
    use std::fs;

    fn write_bytes(path: &std::path::Path, bytes: &[u8]) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, bytes).unwrap();
    }

    #[test]
    fn inspect_import_drop_splits_files_and_flattens_nested_groups() {
        let root = std::env::temp_dir().join(format!("sn-import-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        write_bytes(&root.join("loose.mp3"), b"mp3");
        write_bytes(&root.join("skip.txt"), b"nope");
        let folder = root.join("Board");
        write_bytes(&folder.join("root.wav"), b"wav");
        fs::create_dir_all(folder.join("empty")).unwrap();
        write_bytes(&folder.join("Sub").join("nested.ogg"), b"ogg");
        write_bytes(&folder.join("Sub").join("deep").join("deep.mp3"), b"mp3");

        let scan = inspect_import_drop(
            vec![
                root.join("loose.mp3").to_string_lossy().into_owned(),
                root.join("skip.txt").to_string_lossy().into_owned(),
                folder.to_string_lossy().into_owned(),
            ],
            vec!["mp3".into(), "wav".into(), "ogg".into()],
        )
        .unwrap();
        let _ = fs::remove_dir_all(&root);

        assert_eq!(scan.files.len(), 1);
        assert!(scan.files[0].file_name.eq_ignore_ascii_case("loose.mp3"));
        assert_eq!(scan.skipped, 1);
        assert_eq!(scan.folders.len(), 1);
        assert_eq!(scan.folders[0].name, "Board");
        assert_eq!(scan.folders[0].root_audio.len(), 1);
        assert_eq!(scan.folders[0].groups.len(), 1);
        assert_eq!(scan.folders[0].groups[0].name, "Sub");
        assert_eq!(scan.folders[0].groups[0].audio.len(), 2);
    }

    #[test]
    fn inspect_import_drop_empty_folder_counts_as_skipped() {
        let dir = std::env::temp_dir().join(format!("sn-import-empty-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let scan = inspect_import_drop(
            vec![dir.to_string_lossy().into_owned()],
            vec!["mp3".into()],
        )
        .unwrap();
        let _ = fs::remove_dir_all(&dir);
        assert!(scan.files.is_empty());
        assert!(scan.folders.is_empty());
        assert_eq!(scan.skipped, 1);
    }

    #[test]
    fn list_image_files_abs_returns_matching_full_paths() {
        let dir = std::env::temp_dir().join(format!("sn-img-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("a.gif"), b"GIF89a").unwrap();
        fs::write(dir.join("b.txt"), b"nope").unwrap();
        fs::write(dir.join("c.PNG"), b"png").unwrap();
        let listed = list_image_files_abs(dir.to_string_lossy().into_owned()).unwrap();
        let _ = fs::remove_dir_all(&dir);
        assert_eq!(listed.len(), 2);
        let lower: Vec<String> = listed.iter().map(|p| p.replace('\\', "/").to_lowercase()).collect();
        assert!(lower.iter().any(|p| p.ends_with("/a.gif")));
        assert!(lower.iter().any(|p| p.ends_with("/c.png")));
    }

    #[test]
    fn list_image_files_abs_missing_dir_is_empty() {
        let listed = list_image_files_abs(String::from("/this/path/does/not/exist-sn-test")).unwrap();
        assert!(listed.is_empty());
    }
}
