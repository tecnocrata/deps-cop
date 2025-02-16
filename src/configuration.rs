// use serde::{Deserialize, Serialize};
// use figment::{Figment, providers::{Format, Json, Serialized}};
// use std::{collections::HashMap, path::PathBuf};
// use regex::Regex;
// use glob::Pattern;

// #[derive(Serialize, Deserialize, Debug, Default)]
// pub struct Colors {
//     pub colors: HashMap<String, String>,
// }

// #[derive(Serialize, Deserialize, Debug, Default)]
// pub struct Rules {
//     pub rules: HashMap<String, Vec<String>>,
// }

// #[derive(Serialize, Deserialize, Debug, Default)]
// pub struct Toggles {
//     pub show_valid_dependencies: bool,
//     pub show_invalid_dependencies: bool,
//     pub show_recognized_nodes: bool,
//     pub show_unrecognized_nodes: bool,
// }

// #[derive(Serialize, Deserialize, Debug, Default)]
// pub struct Global {
//     pub layers: Vec<String>,
//     pub colors: HashMap<String, String>,
//     pub rules: HashMap<String, Vec<String>>,
//     pub toggles: Toggles,
// }

// #[derive(Serialize, Deserialize, Debug, Default)]
// pub struct Exclude {
//     pub folders: Vec<String>,
//     pub projects: Vec<String>,
//     pub namespaces: Vec<String>,
//     pub files: Vec<String>,
// }

// #[derive(Serialize, Deserialize, Debug, Clone)]
// #[serde(untagged)]
// pub enum StringOrVec {
//     String(String),
//     Vec(Vec<String>),
// }

// #[derive(Serialize, Deserialize, Debug, Default)]
// pub struct Csharp {
//     pub pattern: String,
//     pub case_sensitive: bool,
//     pub exclude: Exclude,
//     pub projects: HashMap<String, StringOrVec>,
//     pub namespaces: HashMap<String, StringOrVec>,
// }

// #[derive(Serialize, Deserialize, Debug, Default)]
// pub struct Javascript {
//     pub pattern: String,
//     pub case_sensitive: bool,
//     pub exclude: Exclude,
//     pub folders: HashMap<String, StringOrVec>,
// }

// #[derive(Serialize, Deserialize, Debug)]
// pub struct Config {
//     pub global: Global,
//     pub csharp: Option<Csharp>,
//     pub javascript: Option<Javascript>,
// }

// impl Config {
//     pub fn get_color(&self, layer: &str) -> Option<&String> {
//         self.global.colors.get(layer)
//     }
// }

// impl Default for Config {
//     fn default() -> Self {
//         let mut colors = HashMap::new();
//         colors.insert("core".to_string(), "#FBFDB8".to_string());
//         colors.insert("io".to_string(), "#A7D7FD".to_string());
//         colors.insert("usecase".to_string(), "#FEA29C".to_string());

//         let mut rules = HashMap::new();
//         rules.insert("core".to_string(), vec!["core".to_string()]);
//         rules.insert("io".to_string(), vec!["core".to_string(), "io".to_string(), "usecase".to_string()]);
//         rules.insert("usecase".to_string(), vec!["core".to_string(), "usecase".to_string()]);

//         Self {
//             global: Global {
//                 layers: vec!["core".to_string(), "io".to_string(), "usecase".to_string()],
//                 colors: colors,
//                 rules: rules,
//                 toggles: Toggles {
//                     show_valid_dependencies: true,
//                     show_invalid_dependencies: true,
//                     show_recognized_nodes: true,
//                     show_unrecognized_nodes: true,
//                 },
//             },
//             csharp: Some(Csharp {
//                 pattern: "regex".to_string(),
//                 case_sensitive: true,
//                 exclude: Exclude {
//                     folders: vec!["bin".to_string(), "obj".to_string()],
//                     projects: vec![],
//                     namespaces: vec![],
//                     files: vec![],
//                 },
//                 projects: [
//                     ("core".to_string(), StringOrVec::String(r".*\.Entities.*\.csproj$".to_string())),
//                     ("io".to_string(), StringOrVec::String(r".*\.IO.*\.csproj$".to_string())),
//                     ("usecase".to_string(), StringOrVec::String(r".*\.UseCases.*\.csproj$".to_string())),
//                 ]
//                 .iter()
//                 .cloned()
//                 .collect(),
//                 namespaces: [
//                     ("core".to_string(), StringOrVec::String(".*\\.Entities(\\..*)?$".to_string())),
//                     ("io".to_string(), StringOrVec::String(".*\\.IO(\\..*)?$".to_string())),
//                     ("usecase".to_string(), StringOrVec::String(".*\\.UseCases(\\..*)?$".to_string())),
//                 ]
//                 .iter()
//                 .cloned()
//                 .collect(),
//             }),
//             javascript: Some(Javascript {
//                 pattern: "wildcard".to_string(),
//                 case_sensitive: false,
//                 exclude: Exclude {
//                     folders: vec!["node_modules".to_string()],
//                     projects: vec![],
//                     namespaces: vec![],
//                     files: vec![],
//                 },
//                 folders: [
//                     ("core".to_string(), StringOrVec::String("*Entities*".to_string())),
//                     ("io".to_string(), StringOrVec::String("*IO*".to_string())),
//                     ("usecase".to_string(), StringOrVec::String("*UseCase*".to_string())),
//                 ]
//                 .iter()
//                 .cloned()
//                 .collect(),
//             }),
//         }
//     }
// }

// pub fn load_config(project_path: &PathBuf) -> Config {
//     let config_path = project_path.join("depscoprc.json");
//     Figment::from(Serialized::defaults(Config::default()))
//         .merge(Json::file(&config_path))
//         .extract()
//         .unwrap_or_default()
// }

// pub fn determine_layer(name: &str, layer_configs: &HashMap<String, StringOrVec>, case_sensitive: bool, pattern_type: &str) -> String {
//     for (layer, pattern) in layer_configs {
//         let patterns = match pattern {
//             StringOrVec::String(p) => vec![p.clone()],
//             StringOrVec::Vec(ps) => ps.clone(),
//         };

//         for pat in patterns {
//             let pat = if !case_sensitive {
//                 pat.to_lowercase()
//             } else {
//                 pat
//             };

//             match pattern_type {
//                 "regex" => {
//                     if let Ok(re) = Regex::new(&pat) {
//                         if re.is_match(name) {
//                             return layer.clone();
//                         }
//                     }
//                 }
//                 "wildcard" => {
//                     if let Ok(glob) = Pattern::new(&pat) {
//                         if glob.matches(name) {
//                             return layer.clone();
//                         }
//                     }
//                 }
//                 _ => {}
//             }
//         }
//     }
//     "unknown".to_string()
// }

// fn matches_pattern(item: &str, patterns: &[String], pattern_type: &str, case_sensitive: bool) -> bool {
//     for pat in patterns {
//         let pat = if !case_sensitive { pat.to_lowercase() } else { pat.clone() };
//         match pattern_type {
//             "regex" => {
//                 if let Ok(re) = Regex::new(&pat) {
//                     if re.is_match(item) {
//                         return true;
//                     }
//                 }
//             }
//             "wildcard" => {
//                 if let Ok(glob) = Pattern::new(&pat) {
//                     if glob.matches(item) {
//                         return true;
//                     }
//                 }
//             }
//             _ => {}
//         }
//     }
//     false
// }

// pub fn exclude_files_and_folders(path: &PathBuf, exclude: &Exclude, pattern_type: &str, case_sensitive: bool) -> bool {
//     let path_str = match path.to_str() {
//         Some(p) => p,
//         None => return false,
//     };
//     matches_pattern(path_str, &exclude.folders, pattern_type, case_sensitive) ||
//         matches_pattern(path_str, &exclude.files, pattern_type, case_sensitive)
// }

// pub fn exclude_namespaces(namespace: &str, exclude: &Exclude, pattern_type: &str, case_sensitive: bool) -> bool {
//     matches_pattern(namespace, &exclude.namespaces, pattern_type, case_sensitive)
// }

// pub fn exclude_projects(project_name: &str, exclude: &Exclude, pattern_type: &str, case_sensitive: bool) -> bool {
//     matches_pattern(project_name, &exclude.projects, pattern_type, case_sensitive)
// }
