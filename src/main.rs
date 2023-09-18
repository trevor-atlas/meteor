mod browser;
use browser::HistorySchema;
use glob::glob;
use rusqlite::{Connection, Result};
use std::env;
use std::fs;
use std::time::SystemTime;

// if a browser is running you cannot read the history sqlite directly
// because it's locked. You have to copy it somewhere else and use that copy instead
fn copy_browser_sqlite_to_tmpdir(from: &str, name: &str) {
    let mut dest_path = env::temp_dir();
    dest_path.push(format!("meteor-history-{}.sqlite", name));

    match fs::copy(from, &dest_path) {
        Ok(_) => {
            println!("Copied to {:?}", dest_path)
        }
        Err(e) => {
            println!("Error: {:?}", e)
        }
    };
}

fn prep_browser_sqlite_for_collation(configs: &Vec<browser::HistorySchema>) {
    for config in configs {
        if config.path.contains("*") {
            for (i, entry) in glob(&config.path)
                .expect("Failed to read glob pattern")
                .enumerate()
            {
                match entry {
                    Ok(path) => {
                        copy_browser_sqlite_to_tmpdir(
                            &path.to_str().unwrap(),
                            &format!("{}-{}", &config.browser.to_str(), i + 1),
                        );
                    }
                    Err(e) => println!("{:?}", e),
                }
            }
        }
        copy_browser_sqlite_to_tmpdir(&config.path, config.browser.to_str());
    }
}

fn get_copied_sqlite_paths_for_history_schema(schema: &HistorySchema) -> Vec<String> {
    let mut tmp_path = env::temp_dir();
    tmp_path.push(format!(
        "meteor-history-{}*.sqlite",
        schema.browser.to_str()
    ));
    let final_tmp_path = &tmp_path.to_str().unwrap();

    glob(&final_tmp_path)
        .expect("error finding sqlite globs in tmp dir")
        .filter_map(|path| match path {
            Ok(p) => Some(p.to_string_lossy().to_string()),
            Err(_e) => None,
        })
        .collect::<Vec<String>>()
}

fn run_schema_query(path: &str, schema: &HistorySchema) -> Option<Vec<browser::HistoryEntry>> {
    let connection = match Connection::open(path) {
        Ok(con) => con,
        Err(e) => {
            println!("Error connecting to db: {:?}", e);
            return None;
        }
    };

    let mut statement = match connection.prepare(&schema.query) {
        Ok(val) => val,
        Err(e) => {
            println!("error running query '{:?}' for {:?}", e, schema.browser);
            return None;
        }
    };

    let rows = match statement.query_map([], |row| {
        Ok(browser::HistoryEntry {
            id: format!(
                "{}-{}",
                schema.browser.to_str(),
                row.get_unwrap::<usize, i64>(0)
            ),
            browser: schema.browser.clone(),
            url: row.get_unwrap(1),
            title: row.get_unwrap(2),
            visit_count: row.get_unwrap(3),
            typed_count: row.get_unwrap(4),
            last_visit_time: row.get_unwrap(5),
        })
    }) {
        Ok(rows) => rows.collect::<Result<Vec<_>, _>>(),
        Err(e) => {
            println!("error for {} data row {}", schema.browser.to_str(), e);
            return None;
        }
    };

    match rows {
        Ok(rows) => Some(rows),
        Err(e) => {
            println!("error for {} data row {}", schema.browser.to_str(), e);
            None
        }
    }
}

fn collate_browser_history_data() -> Vec<browser::HistoryEntry> {
    let schemas: Vec<browser::HistorySchema> = browser::Browser::variants()
        .iter()
        .filter_map(|b| browser::get_browser_history_schema(b))
        .collect();

    prep_browser_sqlite_for_collation(&schemas);

    schemas
        .iter()
        .map(|schema| {
            get_copied_sqlite_paths_for_history_schema(schema)
                .iter()
                .filter_map(|path| run_schema_query(path, schema))
                .flatten()
                .collect::<Vec<browser::HistoryEntry>>()
        })
        .flatten()
        .collect::<Vec<browser::HistoryEntry>>()
}

fn calculate_frecency(history: &browser::HistoryEntry) -> f64 {
    let currentTime = std::time::UNIX_EPOCH.elapsed().unwrap().as_secs();
    // Set the weights for the different factors that contribute to the frecency score
    let visit_weight = 0.80;
    let typed_weight = 0.10;
    let age_weight = 0.10;
    let age = if history.last_visit_time > currentTime as i64 {
        0
    } else {
        (currentTime as u128 - history.last_visit_time as u128) / (1 * 60 * 60 * 24)
    };

    // default visit count and typed count to -1 if they are undefined, null or 0
    let visitCount = history.visit_count;
    let typedCount = history.typed_count;

    if visitCount <= 0 || typedCount <= 0 {
        return 0.0;
    }

    // Calculate the frecency score using the weights and age
    let score = (visitCount as f64 * visit_weight) + (typedCount as f64 * typed_weight)
        - (age as f64 * age_weight);
    if score <= 0.0 {
        return 0.0;
    }
    score
}

fn main() {
    let history_entries = collate_browser_history_data();

    let mut new = history_entries
        .iter()
        .filter_map(|history| {
            let score = calculate_frecency(history);
            if score == 0.0 {
                return None;
            }
            Some(history)
        })
        .collect::<Vec<&browser::HistoryEntry>>();

    new.sort_by(|a, b| {
        calculate_frecency(b)
            .partial_cmp(&calculate_frecency(a))
            .unwrap()
    });

    for (i, hist) in new.iter().enumerate() {
        println!("\n{:?}", hist);
        if i >= 10 {
            break;
        }
    }
}
