use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Default)]
struct Feature {
    name: String,
    status: String, // "stable", "nightly", or "unknown"
}

const FEATURES_FILE: &str = "features.json";

fn main() {
    let mut args: Vec<String> = std::env::args().collect();
    // Support invocation as a cargo subcommand: skip the first arg if it's "whats-stable"
    if args.len() > 1 && args[1] == "whats-stable" {
        args.remove(1);
    }
    if args.len() < 2 {
        print_usage();
        return;
    }
    match args[1].as_str() {
        "add" => {
            if args.len() < 3 {
                eprintln!("Usage: add <feature>");
                return;
            }
            add_feature(&args[2]);
        }
        "remove" => {
            if args.len() < 3 {
                eprintln!("Usage: remove <feature>");
                return;
            }
            remove_feature(&args[2]);
        }
        "check" => {
            check_features();
        }
        "list" => {
            list_features();
        }
        _ => print_usage(),
    }
}

fn print_usage() {
    println!("Usage: <add|remove|check|list> [feature]");
}

fn add_feature(feature: &str) {
    let mut features = load_features();
    if features.iter().any(|f| f.name == feature) {
        println!("Feature '{}' already exists.", feature);
        return;
    }
    features.push(Feature {
        name: feature.to_string(),
        status: "unknown".to_string(),
    });
    save_features(&features);
    println!("Added feature '{}'.", feature);
}

fn remove_feature(feature: &str) {
    let mut features = load_features();
    let len_before = features.len();
    features.retain(|f| f.name != feature);
    if features.len() == len_before {
        println!("Feature '{}' not found.", feature);
    } else {
        save_features(&features);
        println!("Removed feature '{}'.", feature);
    }
}

fn check_features() {
    let mut features = load_features();
    // Try to locate rust-src for the current toolchain
    let rustup_home = std::env::var("RUSTUP_HOME").unwrap_or_else(|_| {
        // Default location for rustup on Windows
        format!(
            "{}\\.rustup",
            std::env::var("USERPROFILE").unwrap_or_default()
        )
    });
    let toolchain = std::env::var("RUSTUP_TOOLCHAIN")
        .unwrap_or_else(|_| "stable-x86_64-pc-windows-msvc".to_string());
    let rust_src_path = format!(
        "{}\\toolchains\\{}\\lib\\rustlib\\src\\rust\\compiler\\rustc_feature\\src",
        rustup_home, toolchain
    );
    let accepted_path = format!("{}\\accepted.rs", rust_src_path);
    let active_path = format!("{}\\active.rs", rust_src_path);
    let accepted = std::fs::read_to_string(&accepted_path).ok();
    let active = std::fs::read_to_string(&active_path).ok();

    for feature in &mut features {
        // 1. Check unstable book (nightly)
        let url = format!(
            "https://doc.rust-lang.org/nightly/unstable-book/language-features/{}.html",
            feature.name
        );
        let resp = ureq::get(&url).call();
        if resp.is_ok() {
            feature.status = "nightly".to_string();
        } else if let (Some(accepted), Some(active)) = (accepted.as_ref(), active.as_ref()) {
            // 2. Check rust-src registry if available
            if accepted.contains(&format!("\"{}\"", feature.name)) {
                feature.status = "stable".to_string();
            } else if active.contains(&format!("\"{}\"", feature.name)) {
                feature.status = "nightly".to_string();
            } else {
                feature.status = "unknown".to_string();
            }
        } else {
            // 3. Could not check registry, mark as unknown
            feature.status = "unknown".to_string();
        }
        println!("{}: {}", feature.name, feature.status);
    }
    save_features(&features);
    println!("Checked all features.");
    if accepted.is_none() || active.is_none() {
        println!(
            "Note: For more accurate results, install rust-src with 'rustup component add rust-src'."
        );
    }
}

fn list_features() {
    let features = load_features();
    if features.is_empty() {
        println!("No features tracked.");
        return;
    }
    for feature in features {
        println!("{}: {}", feature.name, feature.status);
    }
}

fn load_features() -> Vec<Feature> {
    if !Path::new(FEATURES_FILE).exists() {
        return Vec::new();
    }
    let data = fs::read_to_string(FEATURES_FILE).unwrap_or_default();
    serde_json::from_str(&data).unwrap_or_else(|_| Vec::new())
}

fn save_features(features: &Vec<Feature>) {
    let data = serde_json::to_string_pretty(features).unwrap();
    let mut file = fs::File::create(FEATURES_FILE).unwrap();
    file.write_all(data.as_bytes()).unwrap();
}
