use quicli::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json;
use std::env::var;

use std::fs;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BitbucketConfig {
    pub app_id: String,
    pub app_password: String,
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GitlabConfig {
    pub token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GithubConfig {
    pub user: String,
    pub token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub gitlab: Option<GitlabConfig>,
    pub bitbucket: Option<BitbucketConfig>,
    pub github: Option<GithubConfig>,
}

#[derive(Debug)]
pub enum LoadError {
    FileNotFound,
    HomeEnvNotFound,
    ParseError,
}

pub fn load() -> Result<Config, LoadError> {
    let home = match var("HOME") {
        Ok(ok) => Ok(ok),
        Err(e) => {
            error!("HOME env not found, {}", e);
            Err(LoadError::HomeEnvNotFound)
        }
    }?;

    // let filename = format!("{}/.private/gclone.json", home); TODO - make it configurable ;)
    let filename = format!("{}/next/.private/gclone.json", home);
    let file_result: Result<String, LoadError> = match fs::read_to_string(filename) {
        Ok(s) => Ok(s),
        _error => Err(LoadError::FileNotFound),
    };

    let file_content = file_result?;

    match serde_json::from_str::<Config>(&file_content) {
        Ok(s) => Ok(s),
        _error => Err(LoadError::ParseError),
    }
}
