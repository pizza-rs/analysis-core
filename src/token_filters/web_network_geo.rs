use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

// ═══════════════════════════════════════════════════════════════════════════════
// WEB, NETWORK & GEO FILTERS — URL, IP, domain, coordinate intelligence
// ═══════════════════════════════════════════════════════════════════════════════

/// Extracts domain from a URL token: "https://www.example.com/path" → "example.com"
#[derive(Clone, Debug)]
pub struct DomainExtractTokenFilter {
    pub include_subdomain: bool,
}
impl DomainExtractTokenFilter {
    pub fn new() -> Self {
        Self {
            include_subdomain: false,
        }
    }
}
impl Default for DomainExtractTokenFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenFilter for DomainExtractTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if let Some(domain) = extract_domain(text, self.include_subdomain) {
            token.term = Cow::Owned(domain);
        }
        (false, None)
    }
}

/// Extracts the TLD (top-level domain): "www.example.co.uk" → "co.uk"
#[derive(Clone, Debug)]
pub struct TldExtractTokenFilter;
impl TldExtractTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for TldExtractTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for TldExtractTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if let Some(tld) = extract_tld(text) {
            let tag = Token {
                term: Cow::Owned(format!("_tld:{}", tld)),
                start_offset: token.start_offset,
                end_offset: token.end_offset,
                position: token.position,
            };
            return (false, Some(alloc::vec![tag]));
        }
        (false, None)
    }
}

/// Extracts URL scheme: "https://..." → emits "_scheme:https"
#[derive(Clone, Debug)]
pub struct UrlSchemeTokenFilter;
impl UrlSchemeTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for UrlSchemeTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for UrlSchemeTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if let Some(pos) = text.find("://") {
            let scheme = &text[..pos];
            let tag = Token {
                term: Cow::Owned(format!("_scheme:{}", scheme)),
                start_offset: token.start_offset,
                end_offset: token.end_offset,
                position: token.position,
            };
            return (false, Some(alloc::vec![tag]));
        }
        (false, None)
    }
}

/// Extracts URL path segments as separate tokens.
#[derive(Clone, Debug)]
pub struct UrlPathTokenFilter;
impl UrlPathTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for UrlPathTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for UrlPathTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let path_start = if let Some(pos) = text.find("://") {
            text[pos + 3..].find('/').map(|p| pos + 3 + p)
        } else if text.starts_with('/') {
            Some(0)
        } else {
            None
        };

        if let Some(start) = path_start {
            let path = &text[start..];
            let path = path.split('?').next().unwrap_or(path);
            let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
            if segments.is_empty() {
                return (false, None);
            }
            let extras: Vec<Token<'a>> = segments
                .iter()
                .map(|seg| Token {
                    term: Cow::Owned(String::from(*seg)),
                    start_offset: token.start_offset,
                    end_offset: token.end_offset,
                    position: token.position,
                })
                .collect();
            return (false, Some(extras));
        }
        (false, None)
    }
}

/// Normalizes IP addresses to zero-padded form for lexicographic sorting.
/// "192.168.1.1" → "192.168.001.001"
#[derive(Clone, Debug)]
pub struct IpNormalizationTokenFilter;
impl IpNormalizationTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for IpNormalizationTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for IpNormalizationTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let parts: Vec<&str> = text.split('.').collect();
        if parts.len() == 4 && parts.iter().all(|p| p.parse::<u8>().is_ok()) {
            let normalized = parts
                .iter()
                .map(|p| format!("{:03}", p.parse::<u8>().unwrap()))
                .collect::<Vec<_>>()
                .join(".");
            token.term = Cow::Owned(normalized);
        }
        (false, None)
    }
}

/// Converts IP to a numeric value for range queries.
#[derive(Clone, Debug)]
pub struct IpToNumericTokenFilter;
impl IpToNumericTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for IpToNumericTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for IpToNumericTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let parts: Vec<&str> = text.split('.').collect();
        if parts.len() == 4 {
            let octets: Option<Vec<u32>> = parts.iter().map(|p| p.parse::<u32>().ok()).collect();
            if let Some(o) = octets {
                if o.iter().all(|&v| v <= 255) {
                    let num = (o[0] << 24) | (o[1] << 16) | (o[2] << 8) | o[3];
                    let synonym = Token {
                        term: Cow::Owned(format!("{}", num)),
                        start_offset: token.start_offset,
                        end_offset: token.end_offset,
                        position: token.position,
                    };
                    return (false, Some(alloc::vec![synonym]));
                }
            }
        }
        (false, None)
    }
}

/// Tags IP addresses with their class/type.
#[derive(Clone, Debug)]
pub struct IpClassifyTokenFilter;
impl IpClassifyTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for IpClassifyTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for IpClassifyTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let parts: Vec<&str> = text.split('.').collect();
        if parts.len() == 4 {
            if let (Ok(a), Ok(b)) = (parts[0].parse::<u8>(), parts[1].parse::<u8>()) {
                let class = classify_ip(a, b);
                let tag = Token {
                    term: Cow::Owned(format!("_ip_class:{}", class)),
                    start_offset: token.start_offset,
                    end_offset: token.end_offset,
                    position: token.position,
                };
                return (false, Some(alloc::vec![tag]));
            }
        }
        (false, None)
    }
}

/// Converts coordinates to geohash: "40.7128,-74.0060" → geohash
#[derive(Clone, Debug)]
pub struct GeohashTokenFilter {
    pub precision: usize,
}
impl GeohashTokenFilter {
    pub fn new(precision: usize) -> Self {
        Self { precision }
    }
}
impl Default for GeohashTokenFilter {
    fn default() -> Self {
        Self::new(6)
    }
}

impl TokenFilter for GeohashTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        // Try to parse "lat,lng" format
        let parts: Vec<&str> = text.split(',').collect();
        if parts.len() == 2 {
            if let (Ok(lat), Ok(lng)) = (
                parts[0].trim().parse::<f64>(),
                parts[1].trim().parse::<f64>(),
            ) {
                if (-90.0..=90.0).contains(&lat) && (-180.0..=180.0).contains(&lng) {
                    let hash = encode_geohash(lat, lng, self.precision);
                    token.term = Cow::Owned(hash);
                    return (false, None);
                }
            }
        }
        (false, None)
    }
}

/// Emits geohash prefixes at multiple precision levels for hierarchical search.
#[derive(Clone, Debug)]
pub struct GeohashPrefixTokenFilter {
    pub min_precision: usize,
    pub max_precision: usize,
}
impl GeohashPrefixTokenFilter {
    pub fn new(min: usize, max: usize) -> Self {
        Self {
            min_precision: min,
            max_precision: max,
        }
    }
}
impl Default for GeohashPrefixTokenFilter {
    fn default() -> Self {
        Self::new(2, 6)
    }
}

impl TokenFilter for GeohashPrefixTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        // Assume input is already a geohash
        if text.len() >= self.max_precision && text.chars().all(|c| GEOHASH_BASE32.contains(c)) {
            let mut extras = Vec::new();
            for prec in self.min_precision..text.len() {
                extras.push(Token {
                    term: Cow::Owned(String::from(&text[..prec])),
                    start_offset: token.start_offset,
                    end_offset: token.end_offset,
                    position: token.position,
                });
            }
            return (false, Some(extras));
        }
        (false, None)
    }
}

/// Normalizes URLs: removes www, trailing slash, lowercases scheme+host.
#[derive(Clone, Debug)]
pub struct UrlNormalizationTokenFilter {
    pub remove_www: bool,
    pub remove_trailing_slash: bool,
    pub remove_fragment: bool,
}
impl UrlNormalizationTokenFilter {
    pub fn new() -> Self {
        Self {
            remove_www: true,
            remove_trailing_slash: true,
            remove_fragment: true,
        }
    }
}
impl Default for UrlNormalizationTokenFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenFilter for UrlNormalizationTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let mut text = String::from(token.term.as_ref());

        // Remove fragment
        if self.remove_fragment {
            if let Some(pos) = text.find('#') {
                text.truncate(pos);
            }
        }
        // Lowercase scheme and host
        if let Some(pos) = text.find("://") {
            let (scheme_host, rest) = if let Some(path_pos) = text[pos + 3..].find('/') {
                text.split_at(pos + 3 + path_pos)
            } else {
                (text.as_str(), "")
            };
            text = format!("{}{}", scheme_host.to_lowercase(), rest);
        }
        // Remove www.
        if self.remove_www {
            text = text.replace("://www.", "://");
        }
        // Remove trailing slash
        if self.remove_trailing_slash && text.ends_with('/') && text.matches('/').count() > 3 {
            text.pop();
        }

        if text != token.term.as_ref() {
            token.term = Cow::Owned(text);
        }
        (false, None)
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────

fn extract_domain(url: &str, include_subdomain: bool) -> Option<String> {
    let without_scheme = if let Some(pos) = url.find("://") {
        &url[pos + 3..]
    } else {
        url
    };
    let host = without_scheme.split('/').next()?;
    let host = host.split(':').next()?; // Remove port
    let host = host.split('?').next()?; // Remove query

    if include_subdomain {
        return Some(String::from(host));
    }

    // Remove subdomain (keep last 2 parts, or 3 for co.uk etc.)
    let parts: Vec<&str> = host.split('.').collect();
    if parts.len() <= 2 {
        Some(String::from(host))
    } else {
        let domain = parts[parts.len() - 2..].join(".");
        Some(domain)
    }
}

fn extract_tld(text: &str) -> Option<&str> {
    let host = if let Some(pos) = text.find("://") {
        text[pos + 3..].split('/').next()?
    } else {
        text.split('/').next()?
    };
    let host = host.split(':').next()?;
    host.rfind('.').map(|pos| &host[pos + 1..])
}

fn classify_ip(a: u8, b: u8) -> &'static str {
    match (a, b) {
        (10, _) => "private",
        (172, 16..=31) => "private",
        (192, 168) => "private",
        (127, _) => "loopback",
        (169, 254) => "link_local",
        (224..=239, _) => "multicast",
        (0, _) => "unspecified",
        (255, 255) => "broadcast",
        _ => "public",
    }
}

const GEOHASH_BASE32: &str = "0123456789bcdefghjkmnpqrstuvwxyz";

fn encode_geohash(lat: f64, lng: f64, precision: usize) -> String {
    let base32: Vec<char> = GEOHASH_BASE32.chars().collect();
    let mut result = String::with_capacity(precision);
    let mut lat_range = (-90.0f64, 90.0f64);
    let mut lng_range = (-180.0f64, 180.0f64);
    let mut is_lng = true;
    let mut bit = 0u8;
    let mut ch_idx = 0u32;

    while result.len() < precision {
        if is_lng {
            let mid = (lng_range.0 + lng_range.1) / 2.0;
            if lng >= mid {
                ch_idx |= 1 << (4 - bit);
                lng_range.0 = mid;
            } else {
                lng_range.1 = mid;
            }
        } else {
            let mid = (lat_range.0 + lat_range.1) / 2.0;
            if lat >= mid {
                ch_idx |= 1 << (4 - bit);
                lat_range.0 = mid;
            } else {
                lat_range.1 = mid;
            }
        }
        is_lng = !is_lng;
        bit += 1;
        if bit == 5 {
            result.push(base32[ch_idx as usize]);
            bit = 0;
            ch_idx = 0;
        }
    }
    result
}
