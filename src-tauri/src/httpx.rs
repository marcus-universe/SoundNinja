//! Host-allowlisted HTTP fetches for Klipy / GifSnap (search JSON + GIF bytes).

use base64::{engine::general_purpose::STANDARD, Engine as _};

const MAX_BYTES: usize = 8 * 1024 * 1024;
const MAX_TEXT: usize = 2 * 1024 * 1024;

fn is_gifsnap_r2(host: &str) -> bool {
    // GifSnap media is on Cloudflare R2 public buckets: pub-<id>.r2.dev
    host.starts_with("pub-") && host.ends_with(".r2.dev") && host.bytes().filter(|&b| b == b'.').count() == 2
}

pub(crate) fn host_allowed(host: &str) -> bool {
    let h = host.trim().trim_end_matches('.').to_ascii_lowercase();
    if h.parse::<std::net::IpAddr>().is_ok() {
        return false;
    }
    h == "klipy.com"
        || h.ends_with(".klipy.com")
        || h == "klipycdn.com"
        || h.ends_with(".klipycdn.com")
        || (h.contains("klipy") && !h.contains("localhost"))
        || h == "gifsnap.com"
        || h.ends_with(".gifsnap.com")
        || is_gifsnap_r2(&h)
}

pub(crate) fn parse_https_url(url: &str) -> Result<reqwest::Url, String> {
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

/// GET UTF-8 text (Klipy / GifSnap search JSON) from an allowlisted HTTPS URL.
#[tauri::command]
pub async fn http_get_text(url: String) -> Result<String, String> {
    let bytes = fetch_limited(&url, MAX_TEXT).await?;
    String::from_utf8(bytes).map_err(|_| "Response is not UTF-8".to_string())
}

#[cfg(test)]
mod tests {
    use super::{host_allowed, parse_https_url};

    #[test]
    fn host_allowed_rejects_ip() {
        assert!(!host_allowed("1.2.3.4"));
    }

    #[test]
    fn host_allowed_accepts_klipy() {
        assert!(host_allowed("klipy.com"));
        assert!(host_allowed("cdn.klipy.com"));
    }

    #[test]
    fn host_allowed_accepts_gifsnap() {
        assert!(host_allowed("gifsnap.com"));
        assert!(host_allowed("api.gifsnap.com"));
        assert!(host_allowed("pub-9502c4126a384b90aa92ed45d7f6c379.r2.dev"));
    }

    #[test]
    fn host_allowed_rejects_other_r2() {
        assert!(!host_allowed("r2.dev"));
        assert!(!host_allowed("evil.r2.dev"));
        assert!(!host_allowed("pub-x.other.r2.dev"));
    }

    #[test]
    fn host_allowed_rejects_other_host() {
        assert!(!host_allowed("example.com"));
    }

    #[test]
    fn parse_https_url_rejects_http() {
        assert!(parse_https_url("http://klipy.com/x").is_err());
    }

    #[test]
    fn parse_https_url_rejects_unknown_host() {
        assert!(parse_https_url("https://evil.example/x").is_err());
    }
}
