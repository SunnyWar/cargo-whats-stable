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
    let args: Vec<String> = std::env::args().collect();
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
    for feature in &mut features {
        let url = format!(
            "https://doc.rust-lang.org/nightly/unstable-book/language-features/{}.html",
            feature.name
        );
        let resp = ureq::get(&url).call();
        if resp.is_ok() {
            feature.status = "nightly".to_string();
        } else {
            feature.status = "stable".to_string();
        }
        // Print status for each feature as it is checked
        println!("{}: {}", feature.name, feature.status);
    }
    save_features(&features);
    println!("Checked all features.");
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

// --- Next Feature: HTTP status checking stub ---
// To check feature status, we'll use the 'ureq' crate for HTTP requests.
// Next step: add 'ureq' to Cargo.toml and implement HTTP logic in check_features().
