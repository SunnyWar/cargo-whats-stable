use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::Path;
use walkdir::WalkDir;

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
    use std::path::Path;
    use std::process::Command;
    let mut features = load_features();

    // Find sysroot using rustc --print sysroot
    let sysroot = Command::new("rustc")
        .args(["--print", "sysroot"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string());

    // Compose rust-src path
    let rust_src_root = sysroot.as_ref().map(|sysroot| {
        Path::new(sysroot)
            .join("lib")
            .join("rustlib")
            .join("src")
            .join("rust")
    });

    // Paths for language features
    let (accepted, active) = if let Some(rust_src_root) = &rust_src_root {
        let compiler_path = rust_src_root
            .join("compiler")
            .join("rustc_feature")
            .join("src");
        let accepted_path = compiler_path.join("accepted.rs");
        let active_path = compiler_path.join("active.rs");
        let accepted = std::fs::read_to_string(&accepted_path).ok();
        let active = std::fs::read_to_string(&active_path).ok();
        (accepted, active)
    } else {
        (None, None)
    };

    // Helper: Search for #[unstable(feature = "name")] in all .rs files under library/
    fn search_library_feature(rust_src_root: &Path, feature: &str) -> Option<String> {
        let lib_path = rust_src_root.join("library");
        if !lib_path.exists() {
            return None;
        }
        let pattern = format!("#[unstable(feature = \"{}\"", feature);
        let mut found = None;
        let walker = WalkDir::new(&lib_path).into_iter();
        for entry in walker
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false))
        {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                if content.contains(&pattern) {
                    found = Some(entry.path().display().to_string());
                    break;
                }
            }
        }
        found
    }

    // Determine max width for feature name column
    let max_name_len = features
        .iter()
        .map(|f| f.name.len())
        .max()
        .unwrap_or(10)
        .max("Feature".len());
    let status_col_width = 8.max("Status".len());

    println!(
        "{:<width$}  {:<statw$}",
        "Feature",
        "Status",
        width = max_name_len,
        statw = status_col_width
    );
    println!(
        "{:-<width$}  {:-<statw$}",
        "",
        "",
        width = max_name_len,
        statw = status_col_width
    );

    for feature in &mut features {
        // 1. Check unstable book (nightly language feature)
        let url = format!(
            "https://doc.rust-lang.org/nightly/unstable-book/language-features/{}.html",
            feature.name
        );
        let resp = ureq::get(&url).call();
        if resp.is_ok() {
            feature.status = "nightly".to_string();
        } else if let (Some(accepted), Some(active), Some(rust_src_root)) =
            (accepted.as_ref(), active.as_ref(), rust_src_root.as_ref())
        {
            // 2. Check language features in rustc_feature
            if accepted.contains(&format!("\"{}\"", feature.name)) {
                feature.status = "stable".to_string();
            } else if active.contains(&format!("\"{}\"", feature.name)) {
                feature.status = "nightly".to_string();
            } else if search_library_feature(rust_src_root, &feature.name).is_some() {
                // 3. Check for library features
                feature.status = "nightly".to_string();
            } else {
                feature.status = "unknown".to_string();
            }
        } else if let Some(rust_src_root) = rust_src_root.as_ref() {
            // Only rust-src available, check for library features
            if search_library_feature(rust_src_root, &feature.name).is_some() {
                feature.status = "nightly".to_string();
            } else {
                feature.status = "unknown".to_string();
            }
        } else {
            // Could not check registry, mark as unknown
            feature.status = "unknown".to_string();
        }
        println!(
            "{:<width$}  {:<statw$}",
            feature.name,
            feature.status,
            width = max_name_len,
            statw = status_col_width
        );
    }
    save_features(&features);
    println!("Checked all features.");
    if rust_src_root.is_none() {
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
