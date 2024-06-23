use serde::{Deserialize, Serialize};
use figment::{Figment, providers::{Format, Json, Serialized}};
use std::{collections::HashMap, path::PathBuf};
use regex::Regex;
use glob::Pattern;

#[derive(Serialize, Deserialize, Debug, Default)]
struct Colors {
    core: String,
    io: String,
    usecase: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Allowed {
    core: Vec<String>,
    io: Vec<String>,
    usecase: Vec<String>,
}

impl Allowed {
    pub fn get_layers(&self, layer: &str) -> Option<&Vec<String>> {
        match layer {
            "core" => Some(&self.core),
            "io" => Some(&self.io),
            "usecase" => Some(&self.usecase),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Global {
    colors: Colors,
    pub allowed: Allowed,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum StringOrVec {
    String(String),
    Vec(Vec<String>),
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Csharp {
    pub pattern: String,
    pub case_sensitive: bool,
    pub exclude_folders: Vec<String>,
    pub projects: HashMap<String, StringOrVec>,
    pub namespaces: HashMap<String, StringOrVec>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub global: Global,
    pub csharp: Csharp,
}

impl Config {
    pub fn get_color(&self, layer: &str) -> Option<&String> {
        match layer {
            "core" => Some(&self.global.colors.core),
            "io" => Some(&self.global.colors.io),
            "usecase" => Some(&self.global.colors.usecase),
            _ => None,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            global: Global {
                colors: Colors {
                    core: "#FBFDB8".to_string(),
                    io: "#A7D7FD".to_string(),
                    usecase: "#FEA29C".to_string(),
                },
                allowed: Allowed {
                    core: vec!["core".to_string()],
                    io: vec!["core".to_string(), "io".to_string(), "usecase".to_string()],
                    usecase: vec!["core".to_string(), "usecase".to_string()],
                },
            },
            csharp: Csharp {
                pattern: "regex".to_string(),
                case_sensitive: true,
                exclude_folders: vec!["bin".to_string(), "obj".to_string()],
                projects: [
                    ("core".to_string(), StringOrVec::String(r".*\.Entities.*\.csproj$".to_string())),
                    ("io".to_string(), StringOrVec::String(r".*\.IO.*\.csproj$".to_string())),
                    ("usecase".to_string(), StringOrVec::String(r".*\.UseCases.*\.csproj$".to_string())),
                ]
                .iter()
                .cloned()
                .collect(),
                namespaces: [
                    ("core".to_string(), StringOrVec::String(".*\\.Entities(\\..*)?$".to_string())),
                    ("io".to_string(), StringOrVec::String(".*\\.IO(\\..*)?$".to_string())),
                    ("usecase".to_string(), StringOrVec::String(".*\\.UseCases(\\..*)?$".to_string())),
                ]
                .iter()
                .cloned()
                .collect(),
            },
        }
    }
}

pub fn load_config(project_path: &PathBuf) -> Config {
    let config_path = project_path.join("depscoprc.json");
    Figment::from(Serialized::defaults(Config::default()))
        .merge(Json::file(&config_path))
        .extract()
        .unwrap_or_default()
}

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

fn should_exclude(path: &PathBuf, exclude_folders: &Vec<String>) -> bool {
    for folder in exclude_folders {
        if path.to_str().map_or(false, |p| p.contains(folder)) {
            return true;
        }
    }
    false
}