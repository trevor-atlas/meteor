use dirs::home_dir;

#[derive(Debug, PartialEq, Eq, PartialOrd, Clone, Copy)]
pub enum Browser {
    Arc,
    Chrome,
    Firefox,
    Safari,
    Brave,
    // Opera,
    // Vivaldi,
    // Chromium,
    // Edge,
}

impl Browser {
    pub fn variants() -> &'static [Browser] {
        &[
            Browser::Arc,
            Browser::Chrome,
            Browser::Firefox,
            Browser::Safari,
            Browser::Brave,
            // Browser::Opera,
            // Browser::Edge,
            // Browser::Vivaldi,
            // Browser::Chromium,
        ]
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Arc => "Arc",
            Self::Chrome => "Chrome",
            Self::Firefox => "Firefox",
            Self::Safari => "Safari",
            Self::Brave => "Brave",
            // Self::Opera => "Opera",
            // Self::Edge => "Edge",
            // Self::Vivaldi => "Vivaldi",
            // Self::Chromium => "Chromium",
        }
    }
}

#[derive(Debug)]
pub struct HistoryEntry {
    pub browser: Browser,
    pub id: String,
    pub url: String,
    pub title: String,
    pub visit_count: i64,
    pub typed_count: i64,
    /* timestamp in seconds */
    pub last_visit_time: i64,
}

#[derive(Debug)]
pub struct HistorySchema {
    pub browser: Browser,
    pub path: String,
    pub query: String,
}

static DEFAULT_QUERY: &str = "SELECT id, url, title, visit_count, typed_count, CAST(((CAST(last_visit_time as REAL) - 11644473600000000) / 1000000) as BIGINT) as last_visit_time from urls ORDER BY visit_count DESC";

pub fn get_browser_history_schema(browser: &Browser) -> Option<HistorySchema> {
    let (maybe_path, query) = match browser {
        Browser::Arc => (arc_path(), DEFAULT_QUERY),
        Browser::Chrome => (chrome_path(), DEFAULT_QUERY),
        Browser::Firefox => (firefox_path(), "SELECT id, url, COALESCE(title, \"\"), visit_count, typed, COALESCE(CAST(last_visit_date as INTEGER), 0) FROM moz_places ORDER BY visit_count DESC"),
        Browser::Safari => (safari_path(), "SELECT i.id, i.url, COALESCE(v.title, \"\"), i.visit_count, COALESCE(CAST(i.visit_count_score / 100 as INTEGER), 0), CAST(v.visit_time + 978307200 as BIGINT) as visit_time FROM history_items i LEFT JOIN history_visits v ON i.id = v.history_item ORDER BY i.visit_count DESC"),
        Browser::Brave => (brave_path(), DEFAULT_QUERY),
        // Browser::Opera => (opera_path(), DEFAULT_QUERY),
        // Browser::Edge => (edge_path(), DEFAULT_QUERY),
        // Browser::Vivaldi => (vivaldi_path(), DEFAULT_QUERY),
        // Browser::Chromium => (chromium_path(), DEFAULT_QUERY),
    };

    let home_path_buf = match home_dir() {
        Some(dir) => dir,
        None => panic!("could not get a valid home dir!"),
    };

    let home_path_str = match home_path_buf.as_path().to_str() {
        Some(str_path) => str_path,
        None => panic!("could not parse home dir to string!"),
    };

    match maybe_path {
        Some(p) => {
            let final_path = p.replace("{}", home_path_str);
            Some(HistorySchema {
                browser: *browser,
                path: final_path,
                query: query.to_string(),
            })
        }
        None => None,
    }
}

fn arc_path() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        Some("{}/Library/Application Support/Arc/User Data/Default/History".into())
    }

    #[cfg(target_os = "windows")]
    {
        Some("{}\\AppData\\Local\\Arc\\User Data\\Default\\History".into())
    }

    #[cfg(target_os = "linux")]
    {
        Some("{}/.config/Arc/User Data/Default/History".into())
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        None
    }
}

fn chrome_path() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        Some("{}/Library/Application Support/Google/Chrome/Default/History".into())
    }

    #[cfg(target_os = "windows")]
    {
        Some("{}\\AppData\\Local\\Google\\Chrome\\User Data\\Default\\History".into())
    }

    #[cfg(target_os = "linux")]
    {
        Some("{}/.config/google-chrome/Default/History".into())
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        None
    }
}

fn firefox_path() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        Some("{}/Library/Application Support/Firefox/Profiles/**/places.sqlite".into())
    }

    #[cfg(target_os = "windows")]
    {
        Some(
            "{}\\AppData\\Roaming\\Mozilla\\Firefox\\Profiles\\*.default-release\\places.sqlite"
                .into(),
        )
    }

    #[cfg(target_os = "linux")]
    {
        Some("{}/.mozilla/firefox/*.default-release/places.sqlite".into())
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        None
    }
}

fn safari_path() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        Some("{}/Library/Safari/History.db".into())
    }

    #[cfg(not(target_os = "macos"))]
    {
        None
    }
}

fn opera_path() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        Some("{}/Library/Application Support/com.operasoftware.Opera/History".into())
    }

    #[cfg(target_os = "windows")]
    {
        Some("{}\\AppData\\Roaming\\Opera Software\\Opera Stable\\History".into())
    }

    #[cfg(target_os = "linux")]
    {
        Some("{}/.config/opera/History".into())
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        None
    }
}

fn brave_path() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        Some("{}/Library/Application Support/BraveSoftware/Brave-Browser/Default/History".into())
    }

    #[cfg(target_os = "windows")]
    {
        Some("{}\\AppData\\Local\\BraveSoftware\\Brave-Browser\\User Data\\Default\\History".into())
    }

    #[cfg(target_os = "linux")]
    {
        Some("{}/.config/BraveSoftware/Brave-Browser/Default/History".into())
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        None
    }
}

fn edge_path() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        Some("{}/Library/Application Support/Microsoft Edge/Default/History".into())
    }

    #[cfg(target_os = "windows")]
    {
        Some("{}\\AppData\\Local\\Microsoft\\Edge\\User Data\\Default\\History".into())
    }

    #[cfg(target_os = "linux")]
    {
        Some("{}/.config/microsoft-edge/Default/History".into())
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        None
    }
}

fn vivaldi_path() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        Some("{}/Library/Application Support/Vivaldi/Default/History".into())
    }

    #[cfg(target_os = "windows")]
    {
        Some("{}\\AppData\\Local\\Vivaldi\\User Data\\Default\\History".into())
    }

    #[cfg(target_os = "linux")]
    {
        Some("{}/.config/vivaldi/Default/History".into())
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        None
    }
}

fn chromium_path() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        Some("{}/Library/Application Support/Chromium/Default/History".into())
    }

    #[cfg(target_os = "windows")]
    {
        Some("{}\\AppData\\Local\\Chromium\\User Data\\Default\\History".into())
    }

    #[cfg(target_os = "linux")]
    {
        Some("{}/.config/chromium/Default/History".into())
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        None
    }
}
