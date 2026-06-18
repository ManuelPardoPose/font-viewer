use font_enumeration::{Collection, Font, Style};
use serde::{Deserialize, Serialize};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

#[derive(Serialize)]
struct SerializableFont {
    family_name: String,
    font_name: String,
    path: String,
    style: String,
    weight: f32,
    stretch: f32,
}

impl From<&Font> for SerializableFont {
    fn from(font: &Font) -> Self {
        Self {
            family_name: font.family_name.clone(),
            font_name: font.font_name.clone(),
            path: font.path.to_string_lossy().into_owned(),
            style: match &font.style {
                Style::Normal => "Normal".to_string(),
                Style::Italic => "Italic".to_string(),
                Style::Oblique(angle) => match angle {
                    Some(angle) => format!("Oblique ({:?}°)", angle),
                    None => "Oblique".to_string(),
                },
            },
            weight: font.weight.value(),
            stretch: font.stretch.value(),
        }
    }
}

#[tauri::command]
fn list_fonts() -> Vec<SerializableFont> {
    let font_collection = Collection::new().unwrap();
    font_collection
        .all()
        .map(SerializableFont::from)
        .collect()
}

// The Google Fonts metadata endpoint doesn't send CORS headers, so the
// webview can't fetch it directly. Proxy the request through Rust instead.
#[tauri::command]
async fn fetch_google_fonts_metadata() -> Result<String, String> {
    reqwest::get("https://fonts.google.com/metadata/fonts")
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())
}

#[derive(Deserialize)]
struct DownloadList {
    manifest: Manifest,
}

#[derive(Deserialize)]
struct Manifest {
    #[serde(rename = "fileRefs")]
    file_refs: Vec<FileRef>,
}

#[derive(Deserialize)]
struct FileRef {
    filename: String,
    url: String,
}

// Downloads a family's font files from Google Fonts and installs them into
// Windows for the current user. Returns the installed file paths.
#[tauri::command]
async fn install_google_font(family: String) -> Result<Vec<String>, String> {
    let client = reqwest::Client::new();
    let body = client
        .get("https://fonts.google.com/download/list")
        .query(&[("family", &family)])
        .send()
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;

    // The response is guarded by an XSSI prefix before the JSON body.
    let json = match body.find('{') {
        Some(i) => &body[i..],
        None => return Err("unexpected response from Google Fonts".into()),
    };
    let list: DownloadList = serde_json::from_str(json).map_err(|e| e.to_string())?;

    let mut installed = Vec::new();
    for file in list.manifest.file_refs {
        let lower = file.filename.to_lowercase();
        if !lower.ends_with(".ttf") && !lower.ends_with(".otf") {
            continue;
        }
        let bytes = client
            .get(&file.url)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .bytes()
            .await
            .map_err(|e| e.to_string())?;
        installed.push(install_font_file(&file.filename, &bytes)?);
    }

    if installed.is_empty() {
        return Err("no installable font files found".into());
    }

    #[cfg(windows)]
    notify_font_change();

    Ok(installed)
}

#[cfg(windows)]
fn install_font_file(filename: &str, bytes: &[u8]) -> Result<String, String> {
    use std::os::windows::ffi::OsStrExt;
    use std::path::{Path, PathBuf};
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let base = Path::new(filename)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("invalid font filename")?;

    let local = std::env::var("LOCALAPPDATA").map_err(|e| e.to_string())?;
    let dir = PathBuf::from(local)
        .join("Microsoft")
        .join("Windows")
        .join("Fonts");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let dest = dir.join(base);
    std::fs::write(&dest, bytes).map_err(|e| e.to_string())?;

    // Make the font usable in the running session right away.
    let wide: Vec<u16> = dest
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    unsafe {
        windows_sys::Win32::Graphics::Gdi::AddFontResourceW(wide.as_ptr());
    }

    // Persist the registration so it survives a reboot (per-user, no admin).
    let stem = Path::new(base)
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or(base);
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu
        .create_subkey(r"Software\Microsoft\Windows NT\CurrentVersion\Fonts")
        .map_err(|e| e.to_string())?;
    key.set_value(format!("{} (TrueType)", stem), &dest.to_string_lossy().to_string())
        .map_err(|e| e.to_string())?;

    Ok(dest.to_string_lossy().into_owned())
}

#[cfg(not(windows))]
fn install_font_file(_filename: &str, _bytes: &[u8]) -> Result<String, String> {
    Err("Installing fonts is only supported on Windows".into())
}

#[cfg(windows)]
fn notify_font_change() {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_FONTCHANGE,
    };
    unsafe {
        SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_FONTCHANGE,
            0,
            0,
            SMTO_ABORTIFHUNG,
            1000,
            std::ptr::null_mut(),
        );
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            list_fonts,
            fetch_google_fonts_metadata,
            install_google_font
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
