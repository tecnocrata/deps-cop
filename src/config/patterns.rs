use regex::Regex;
use glob::Pattern;
use std::collections::HashMap;
use std::path::PathBuf;
use crate::config::types::{StringOrVec, Exclude};

pub fn determine_layer(name: &str, layer_configs: &HashMap<String, StringOrVec>, case_sensitive: bool, pattern_type: &str) -> String {
    for (layer, pattern) in layer_configs {
        let patterns = match pattern {
            StringOrVec::String(p) => vec![p.clone()],
            StringOrVec::Vec(ps) => ps.clone(),
        };

        for pat in patterns {
            let pat = if !case_sensitive {
                pat.to_lowercase()
            } else {
                pat
            };

            match pattern_type {
                "regex" => {
                    if let Ok(re) = Regex::new(&pat) {
                        if re.is_match(name) {
                            return layer.clone();
                        }
                    }
                }
                "wildcard" => {
                    if let Ok(glob) = Pattern::new(&pat) {
                        if glob.matches(name) {
                            return layer.clone();
                        }
                    }
                }
                _ => {}
            }
        }
    }
    "unknown".to_string()
}

fn matches_pattern(item: &str, patterns: &[String], pattern_type: &str, case_sensitive: bool) -> bool {
    for pat in patterns {
        let pat = if !case_sensitive { pat.to_lowercase() } else { pat.clone() };
        match pattern_type {
            "regex" => {
                if let Ok(re) = Regex::new(&pat) {
                    if re.is_match(item) {
                        return true;
                    }
                }
            }
            "wildcard" => {
                if let Ok(glob) = Pattern::new(&pat) {
                    if glob.matches(item) {
                        return true;
                    }
                }
            }
            _ => {}
        }
    }
    false
}

pub fn exclude_files_and_folders(path: &PathBuf, exclude: &Exclude, pattern_type: &str, case_sensitive: bool) -> bool {
    let path_str = match path.to_str() {
        Some(p) => p,
        None => return false,
    };
    matches_pattern(path_str, &exclude.folders, pattern_type, case_sensitive) ||
        matches_pattern(path_str, &exclude.files, pattern_type, case_sensitive)
}

pub fn exclude_namespaces(namespace: &str, exclude: &Exclude, pattern_type: &str, case_sensitive: bool) -> bool {
    matches_pattern(namespace, &exclude.namespaces, pattern_type, case_sensitive)
}

pub fn exclude_projects(project_name: &str, exclude: &Exclude, pattern_type: &str, case_sensitive: bool) -> bool {
    matches_pattern(project_name, &exclude.projects, pattern_type, case_sensitive)
}
