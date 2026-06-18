use font_enumeration::{Collection, Font, Style};
use fontdue::{Font as FontdueFont, FontSettings};
use image_hasher::{HashAlg, Hasher, HasherConfig, ImageHash};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use tauri::ipc::Channel;

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

fn cancel_registry() -> &'static Mutex<HashMap<u64, Arc<AtomicBool>>> {
    static REG: OnceLock<Mutex<HashMap<u64, Arc<AtomicBool>>>> = OnceLock::new();
    REG.get_or_init(|| Mutex::new(HashMap::new()))
}

// Caches a font glyph's perceptual hash keyed by path, mtime, and glyph, so
// repeat similarity runs only pay the hashing cost once.
fn hash_cache() -> &'static Mutex<HashMap<String, ImageHash>> {
    static CACHE: OnceLock<Mutex<HashMap<String, ImageHash>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

// Rasterizes a glyph and places it, centered and aspect-preserved, on a square
// canvas so the hash reflects the glyph's shape rather than its raw size.
fn glyph_image(font: &FontdueFont, c: char, px: f32, canvas: u32) -> Option<image::GrayImage> {
    if font.lookup_glyph_index(c) == 0 {
        return None;
    }
    let (metrics, bitmap) = font.rasterize(c, px);
    if metrics.width == 0 || metrics.height == 0 {
        return None;
    }
    let (w, h) = (metrics.width as u32, metrics.height as u32);
    let glyph = image::GrayImage::from_raw(w, h, bitmap)?;
    let scale = canvas as f32 / w.max(h) as f32;
    let nw = ((w as f32 * scale).round() as u32).clamp(1, canvas);
    let nh = ((h as f32 * scale).round() as u32).clamp(1, canvas);
    let resized = image::imageops::resize(&glyph, nw, nh, image::imageops::FilterType::Triangle);
    let mut out = image::GrayImage::new(canvas, canvas);
    let ox = ((canvas - nw) / 2) as i64;
    let oy = ((canvas - nh) / 2) as i64;
    image::imageops::overlay(&mut out, &resized, ox, oy);
    Some(out)
}

// A text font exposes a Unicode cmap. Symbol/pictorial fonts (Wingdings, MT
// Extra, the classic Symbol font, …) only carry a Windows Symbol cmap, so they
// map Latin codepoints to dingbats - comparing those to letterforms is noise.
fn is_text_font(bytes: &[u8]) -> bool {
    match ttf_parser::Face::parse(bytes, 0) {
        Ok(face) => face
            .tables()
            .cmap
            .map(|cmap| cmap.subtables.into_iter().any(|s| s.is_unicode()))
            .unwrap_or(false),
        Err(_) => false,
    }
}

fn glyph_hashes_for(hasher: &Hasher, path: &str, glyphs: &[char]) -> HashMap<char, ImageHash> {
    let mtime = std::fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let mut result = HashMap::new();
    let mut font: Option<FontdueFont> = None;
    let mut tried_load = false;

    for &c in glyphs {
        let key = format!("{path}|{mtime}|{c}");
        if let Some(hash) = hash_cache().lock().unwrap().get(&key).cloned() {
            result.insert(c, hash);
            continue;
        }
        if !tried_load {
            tried_load = true;
            font = std::fs::read(path).ok().and_then(|bytes| {
                if !is_text_font(&bytes) {
                    return None;
                }
                FontdueFont::from_bytes(bytes, FontSettings::default()).ok()
            });
        }
        let Some(f) = font.as_ref() else { continue };
        if let Some(img) = glyph_image(f, c, 128.0, 64) {
            let hash = hasher.hash_image(&image::DynamicImage::ImageLuma8(img));
            hash_cache().lock().unwrap().insert(key, hash.clone());
            result.insert(c, hash);
        }
    }
    result
}

// Averages per-glyph similarity (1 - normalized Hamming distance) over the
// glyphs both fonts share. Returns None when they share no comparable glyph.
fn similarity_score(
    a: &HashMap<char, ImageHash>,
    b: &HashMap<char, ImageHash>,
    min_glyphs: u32,
) -> Option<f32> {
    let mut total = 0.0f32;
    let mut count = 0u32;
    for (c, ha) in a {
        if let Some(hb) = b.get(c) {
            let bits = (ha.as_bytes().len() * 8) as f32;
            if bits == 0.0 {
                continue;
            }
            total += 1.0 - (ha.dist(hb) as f32 / bits);
            count += 1;
        }
    }
    if count < min_glyphs.max(1) {
        None
    } else {
        Some((total / count as f32) * 100.0)
    }
}

// One representative face per installed family, chosen to match the requested
// italic preference and the weight closest to `weight`. Falls back to the
// nearest available face when an exact italic/weight match doesn't exist.
fn representative_faces(weight: f32, italic: bool) -> Vec<(String, String)> {
    let collection = match Collection::new() {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let mut best: HashMap<String, (bool, f32, String)> = HashMap::new();
    for font in collection.all() {
        let family = font.family_name.clone();
        let face_italic = !matches!(font.style, Style::Normal);
        let style_match = face_italic == italic;
        let weight_dist = (font.weight.value() - weight).abs();
        let path = font.path.to_string_lossy().into_owned();
        let better = match best.get(&family) {
            None => true,
            Some((best_match, best_dist, _)) => {
                if style_match != *best_match {
                    style_match
                } else {
                    weight_dist < *best_dist
                }
            }
        };
        if better {
            best.insert(family, (style_match, weight_dist, path));
        }
    }
    best.into_iter().map(|(k, (_, _, p))| (k, p)).collect()
}

#[derive(Clone, Serialize)]
#[serde(tag = "event", content = "data", rename_all = "camelCase")]
enum SimilarityEvent {
    Started { total: usize },
    Result { family: String, score: Option<f32> },
    Done,
    Cancelled,
}

// Streams perceptual-similarity scores between `target` and every other
// installed family over `on_event`. Cancellable via `cancel_similarity`.
#[tauri::command]
fn find_similar_fonts(
    target: String,
    glyphs: String,
    weight: f32,
    italic: bool,
    job_id: u64,
    on_event: Channel<SimilarityEvent>,
) {
    let cancel = Arc::new(AtomicBool::new(false));
    cancel_registry().lock().unwrap().insert(job_id, cancel.clone());

    std::thread::spawn(move || {
        let finish = || {
            cancel_registry().lock().unwrap().remove(&job_id);
        };

        let glyph_vec: Vec<char> = glyphs.chars().collect();
        // A gradient (edge) hash rather than DCT-based pHash: DCT keeps only
        // low frequencies, where blocky pixel fonts look like a generic letter
        // blob and rank close to everything. Gradients capture the edge
        // structure that actually distinguishes typefaces.
        let hasher = HasherConfig::new()
            .hash_size(16, 16)
            .hash_alg(HashAlg::DoubleGradient)
            .to_hasher();

        let faces = representative_faces(weight, italic);
        let Some(target_path) = faces.iter().find(|(f, _)| f == &target).map(|(_, p)| p.clone())
        else {
            let _ = on_event.send(SimilarityEvent::Done);
            finish();
            return;
        };
        let target_hashes = glyph_hashes_for(&hasher, &target_path, &glyph_vec);
        // Require a candidate to share at least half of the target's glyphs, so
        // a font matching on only one or two glyphs can't rank highly by luck.
        let min_glyphs = ((target_hashes.len() as u32 + 1) / 2).max(1);

        let candidates: Vec<(String, String)> =
            faces.into_iter().filter(|(f, _)| f != &target).collect();
        let _ = on_event.send(SimilarityEvent::Started {
            total: candidates.len(),
        });

        for (family, path) in candidates {
            if cancel.load(Ordering::Relaxed) {
                let _ = on_event.send(SimilarityEvent::Cancelled);
                finish();
                return;
            }
            let hashes = glyph_hashes_for(&hasher, &path, &glyph_vec);
            let score = similarity_score(&target_hashes, &hashes, min_glyphs);
            let _ = on_event.send(SimilarityEvent::Result { family, score });
        }

        let _ = on_event.send(SimilarityEvent::Done);
        finish();
    });
}

#[tauri::command]
fn cancel_similarity(job_id: u64) {
    if let Some(cancel) = cancel_registry().lock().unwrap().get(&job_id) {
        cancel.store(true, Ordering::Relaxed);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            list_fonts,
            fetch_google_fonts_metadata,
            install_google_font,
            find_similar_fonts,
            cancel_similarity
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
