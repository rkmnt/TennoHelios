/// market_api.rs — warframe.market REST API client.
///
/// Item list: v2 API (https://api.warframe.market/v2/items) — includes ducat values.
/// Price statistics: v1 API (https://api.warframe.market/v1/items/{slug}/statistics)
///   — v2 statistics endpoint does not exist yet; v1 works and follows redirects.
use std::collections::HashMap;
use std::sync::OnceLock;

use anyhow::Context;
use log::{info, warn};
use serde::Deserialize;

// ── Constants ─────────────────────────────────────────────────────────────────

const BASE_V1: &str = "https://api.warframe.market/v1";
const BASE_V2: &str = "https://api.warframe.market/v2";

// ── Public types ──────────────────────────────────────────────────────────────

/// Prices for a single item resolved from warframe.market.
#[derive(Debug, Clone)]
pub struct ItemPrice {
    pub url_name: String,
    /// Mean average sell price across closed orders in the last 48 h.
    /// `0.0` if statistics are unavailable.
    pub plat_avg_48h: f32,
    /// Void trader ducat value from the v2 API. `0` if unknown.
    pub ducat_value: u32,
}

// ── API response shapes ───────────────────────────────────────────────────────

#[derive(Deserialize)]
struct V2Item {
    slug: String,
    #[serde(default)]
    ducats: u32,
    i18n: V2I18n,
}

#[derive(Deserialize)]
struct V2I18n {
    en: V2I18nEn,
}

#[derive(Deserialize)]
struct V2I18nEn {
    name: String,
}

#[derive(Deserialize)]
struct V2ItemsResponse {
    data: Vec<V2Item>,
}

#[derive(Deserialize)]
struct StatEntry {
    avg_price: Option<f32>,
}

#[derive(Deserialize)]
struct StatsClosed {
    #[serde(rename = "48hours")]
    last_48h: Vec<StatEntry>,
}

#[derive(Deserialize)]
struct StatsPayload {
    statistics_closed: StatsClosed,
}

#[derive(Deserialize)]
struct StatsResponse {
    payload: StatsPayload,
}

// ── Session cache ─────────────────────────────────────────────────────────────

struct CacheEntry {
    slug: String,
    ducat_value: u32,
}

/// lowercase display name → (slug, ducat_value)
static ITEM_CACHE: OnceLock<HashMap<String, CacheEntry>> = OnceLock::new();

async fn item_cache(client: &reqwest::Client) -> anyhow::Result<&'static HashMap<String, CacheEntry>> {
    if let Some(m) = ITEM_CACHE.get() {
        return Ok(m);
    }

    info!("fetching warframe.market v2 item list…");
    let resp: V2ItemsResponse = client
        .get(format!("{BASE_V2}/items"))
        .header("accept", "application/json")
        .send()
        .await
        .context("GET /v2/items network error")?
        .error_for_status()
        .context("GET /v2/items returned error status")?
        .json()
        .await
        .context("GET /v2/items JSON decode failed")?;

    let map: HashMap<String, CacheEntry> = resp
        .data
        .into_iter()
        .map(|it| {
            let key = it.i18n.en.name.to_lowercase();
            let entry = CacheEntry { slug: it.slug, ducat_value: it.ducats };
            (key, entry)
        })
        .collect();

    info!("cached {} tradeable items", map.len());
    let _ = ITEM_CACHE.set(map);
    Ok(ITEM_CACHE.get().expect("just set"))
}

// ── Statistics ────────────────────────────────────────────────────────────────

async fn fetch_avg_price_48h(client: &reqwest::Client, slug: &str) -> anyhow::Result<Option<f32>> {
    let url = format!("{BASE_V1}/items/{slug}/statistics");
    let resp: StatsResponse = client
        .get(&url)
        .header("platform", "pc")
        .header("language", "en")
        .send()
        .await
        .with_context(|| format!("GET statistics for {slug}: network error"))?
        .error_for_status()
        .with_context(|| format!("GET statistics for {slug}: error status"))?
        .json()
        .await
        .with_context(|| format!("GET statistics for {slug}: JSON decode failed"))?;

    let prices: Vec<f32> = resp
        .payload
        .statistics_closed
        .last_48h
        .iter()
        .filter_map(|e| e.avg_price)
        .collect();

    if prices.is_empty() {
        return Ok(None);
    }
    Ok(Some(prices.iter().sum::<f32>() / prices.len() as f32))
}

// ── Public API ────────────────────────────────────────────────────────────────

pub async fn fetch_prices(item_names: &[String]) -> anyhow::Result<Vec<ItemPrice>> {
    let client = reqwest::Client::builder()
        .user_agent("TennoHelios/0.1 (github.com/rkmnt/TennoHelios)")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .context("failed to build reqwest client")?;

    let cache = item_cache(&client).await?;

    let mut results = Vec::with_capacity(item_names.len());

    for name in item_names {
        let key = name.to_lowercase();
        let Some(entry) = cache.get(&key) else {
            warn!("'{name}' not found in warframe.market item list");
            results.push(ItemPrice { url_name: String::new(), plat_avg_48h: 0.0, ducat_value: 0 });
            continue;
        };

        let plat_avg_48h = match fetch_avg_price_48h(&client, &entry.slug).await {
            Ok(Some(avg)) => { info!("{name}: {avg:.1} plat (48h avg)"); avg }
            Ok(None) => { warn!("{name}: no 48h stats"); 0.0 }
            Err(e) => { warn!("{name}: stats error: {e:#}"); 0.0 }
        };

        results.push(ItemPrice {
            url_name: entry.slug.clone(),
            plat_avg_48h,
            ducat_value: entry.ducat_value,
        });
    }

    Ok(results)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    /// Verify that fetch_prices returns same length as input even for bogus names.
    /// Requires network — skipped in offline CI.
    #[tokio::test]
    #[ignore]
    async fn fetch_prices_length_matches_input() {
        let names = vec![
            "Ash Prime Set".to_string(),
            "Completely Bogus Item XYZ".to_string(),
        ];
        let prices = super::fetch_prices(&names).await.unwrap();
        assert_eq!(prices.len(), names.len());
        assert_eq!(prices[1].plat_avg_48h, 0.0);
    }
}
