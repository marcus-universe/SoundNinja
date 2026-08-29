//! Host-allowlisted HTTP fetches for Klipy (search JSON + GIF bytes).

use base64::{engine::general_purpose::STANDARD, Engine as _};

const MAX_BYTES: usize = 8 * 1024 * 1024;
const MAX_TEXT: usize = 2 * 1024 * 1024;

fn host_allowed(host: &str) -> bool {
    let h = host.trim().trim_end_matches('.').to_ascii_lowercase();
    if h.parse::<std::net::IpAddr>().is_ok() {
        return false;
    }
    h == "klipy.com"
        || h.ends_with(".klipy.com")
        || h == "klipycdn.com"
        || h.ends_with(".klipycdn.com")
        || (h.contains("klipy") && !h.contains("localhost"))
}

fn parse_https_url(url: &str) -> Result<reqwest::Url, String> {
    let parsed = reqwest::Url::parse(url.trim()).map_err(|e| e.to_string())?;
    if parsed.scheme() != "https" {
        return Err("Only HTTPS URLs are allowed".into());
    }
    let host = parsed.host_str().ok_or_else(|| "URL has no host".to_string())?;
    if !host_allowed(host) {
        return Err(format!("Host not allowed: {host}"));
    }
    Ok(parsed)
}

fn redirect_policy() -> reqwest::redirect::Policy {
    reqwest::redirect::Policy::custom(|attempt| {
        match attempt.url().host_str() {
            Some(h) if host_allowed(h) => attempt.follow(),
            _ => attempt.error("redirect host not allowed"),
        }
    })
}

async fn fetch_limited(url: &str, max: usize) -> Result<Vec<u8>, String> {
    let parsed = parse_https_url(url)?;
    let client = reqwest::Client::builder()
        .redirect(redirect_policy())
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .get(parsed)
        .header("User-Agent", "SoundNinja/0.5")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }
    if let Some(len) = resp.content_length() {
        if len as usize > max {
            return Err(format!("Response exceeds {max} byte limit"));
        }
    }
    let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
    if bytes.len() > max {
        return Err(format!("Response exceeds {max} byte limit"));
    }
    Ok(bytes.to_vec())
}

/// Downloads bytes from an allowlisted HTTPS URL and returns them as base64.
#[tauri::command]
pub async fn download_url_bytes(url: String) -> Result<String, String> {
    let bytes = fetch_limited(&url, MAX_BYTES).await?;
    Ok(STANDARD.encode(bytes))
}

/// GET UTF-8 text (Klipy search/trending JSON) from an allowlisted HTTPS URL.
#[tauri::command]
pub async fn http_get_text(url: String) -> Result<String, String> {
    let bytes = fetch_limited(&url, MAX_TEXT).await?;
    String::from_utf8(bytes).map_err(|_| "Response is not UTF-8".to_string())
}
