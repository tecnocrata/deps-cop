use figment::{Figment, providers::{Format, Json, Serialized}};
use std::path::PathBuf;

use crate::config::types::Config;

pub fn load_config(project_path: &PathBuf) -> Config {
    let config_path = project_path.join("depscoprc.json");
    Figment::from(Serialized::defaults(Config::default()))
        .merge(Json::file(&config_path))
        .extract()
        .unwrap_or_default()
}
