use crate::config::BitbucketConfig;
use crate::integration::common::*;
use reqwest;
use reqwest::header;
use serde_json::Value;
use std::collections::HashMap;

const FIRST_URL: &str = "https://api.bitbucket.org/2.0/repositories/?role=member&pagelen=100";

#[derive(Debug)]
pub struct CrawlResult {
    pub next_url: Option<String>,
    pub repository_urls: Vec<String>,
}

pub fn get_all(
    config: &BitbucketConfig,
    save_batch: &dyn Fn(&Vec<String>) -> std::io::Result<()>,
) -> Result<Vec<String>, CrawlError> {
    let token = get_token(config)?;

    let mut link_acc: Vec<String> = vec![];

    let mut maybe_next: Option<String> = Some(FIRST_URL.to_owned());
    while let Some(next) = &maybe_next {
        let mut page = fetch_page(next, &token)?;
        link_acc.append(&mut page.repository_urls);
        let _ = save_batch(&page.repository_urls);
        maybe_next = page.next_url;
    }

    Ok(link_acc)
}

fn fetch_page(url: &str, token: &String) -> Result<CrawlResult, CrawlError> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        header::HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
    );

    let client = reqwest::Client::new();

    let mut response = client
        .get(url)
        .headers(headers)
        .send()
        .map(Ok)
        .unwrap_or(Err(CrawlError::FetchError))?;

    let json = response
        .json::<Value>()
        .map(Ok)
        .unwrap_or(Err(CrawlError::ParseError))?;

    parse_response(json)
}

fn parse_response(value: Value) -> Result<CrawlResult, CrawlError> {
    let next_url = match &value["next"] {
        Value::String(nxt) => Some(nxt.to_owned()),
        _ => None,
    };
    let repo_jsons = match &value["values"] {
        Value::Array(arr) => Ok(arr),
        _ => Err(CrawlError::ParseError),
    }?;

    let mut repository_urls: Vec<String> = vec![];

    for repo in repo_jsons {
        match &repo["links"]["clone"] {
            Value::Array(values) => {
                repository_urls.append(&mut parse_clone_links(values.to_vec())?)
            }
            _ => (),
        }
    }

    Ok(CrawlResult {
        next_url,
        repository_urls,
    })
}

fn parse_clone_links(a: Vec<Value>) -> Result<Vec<String>, CrawlError> {
    let mut result: Vec<String> = vec![];

    for v in a.iter() {
        match (&v["name"], &v["href"]) {
            (Value::String(name), Value::String(href)) => {
                if name == "ssh" {
                    result.push(href.to_string());
                }
            }
            _ => (),
        }
    }
    Ok(result)
}

fn get_token(config: &BitbucketConfig) -> Result<String, CrawlError> {
    let client = reqwest::Client::new();

    let mut payload = HashMap::new();
    payload.insert("username", &config.app_id);
    payload.insert("password", &config.app_password);

    let credentials_grant = &"client_credentials".to_owned();
    payload.insert("grant_type", credentials_grant);

    let http_result = client
        .post("https://bitbucket.org/site/oauth2/access_token")
        .form(&payload)
        .basic_auth(&config.client_id, Some(&config.client_secret))
        .send();

    let typed_result = match http_result {
        Ok(e) => Ok(e),
        Err(e) => {
            eprintln!("Falied http request for bitbucket token due to {}", e);
            Err(CrawlError::FetchError)
        }
    };

    let token: Result<serde_json::Value, CrawlError> =
        match typed_result?.json::<serde_json::Value>() {
            Ok(r) => Ok(r),
            Err(e) => {
                eprintln!("Falied http request for bitbucket token due to {}", e);
                Err(CrawlError::ParseError)
            }
        };

    match &token?["access_token"] {
        serde_json::Value::String(s) => Ok(s.to_owned()),
        _ => Err(CrawlError::ParseError),
    }
}
