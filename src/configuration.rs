use serde::{Deserialize, Serialize};
use figment::{Figment, providers::{Format, Json, Serialized}};
use std::path::PathBuf;
// use std::env;

#[derive(Serialize, Deserialize, Debug, Default)]
struct Colors {
    core: String,
    io: String,
    usecase: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct Allowed {
    core: Vec<String>,
    io: Vec<String>,
    usecase: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct Global {
    colors: Colors,
    allowed: Allowed,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
enum StringOrVec {
    String(String),
    Vec(Vec<String>),
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct Csharp {
    pattern: String,
    case_sensitive: String,
    projects: std::collections::HashMap<String, StringOrVec>,
    namespaces: std::collections::HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    global: Global,
    csharp: Csharp,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            global: Global {
                colors: Colors {
                    core: "red".to_string(),
                    io: "green".to_string(),
                    usecase: "blue".to_string(),
                },
                allowed: Allowed {
                    core: vec!["core".to_string()],
                    io: vec!["core".to_string(), "io".to_string(), "usecase".to_string()],
                    usecase: vec!["core".to_string(), "usecase".to_string()],
                },
            },
            csharp: Csharp {
                pattern: "regex".to_string(),
                case_sensitive: "true".to_string(),
                projects: [
                    ("core".to_string(), StringOrVec::String(".*\\.Entities\\..*\\.csproj$".to_string())),
                    ("io".to_string(), StringOrVec::String(".*\\.IO\\..*\\.csproj$".to_string())),
                    ("usecase".to_string(), StringOrVec::String(".*\\.UseCase\\..*\\.csproj$".to_string())),
                ]
                .iter()
                .cloned()
                .collect(),
                namespaces: [
                    ("core".to_string(), ".*\\.Entities\\..*".to_string()),
                    ("io".to_string(), ".*\\.IO\\..*".to_string()),
                    ("usecase".to_string(), ".*\\.UseCase\\..*".to_string()),
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