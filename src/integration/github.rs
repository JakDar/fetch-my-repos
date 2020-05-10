use crate::config::GithubConfig;
use crate::integration::common::*;
use quicli::prelude::*;
use reqwest;
use reqwest::header;
use serde_json::Value;

const API_URL: &str = "https://api.github.com";
const PER_PAGE: i32 = 100;

#[derive(Debug)]
pub struct CrawlResult {
    pub repository_urls: Vec<String>,
}

pub fn get_all(
    config: &GithubConfig,
    save_batch: &dyn Fn(&Vec<String>) -> std::io::Result<()>,
) -> Result<Vec<String>, CrawlError> {
    let mut links_acc: Vec<String> = vec![];

    let mut i = 1;
    let mut fetched = get_page(&config, i)?.repository_urls;
    let _ = save_batch(&fetched);
    links_acc.append(&mut fetched);

    while !fetched.len() == 0 {
        i += 1;
        fetched = get_page(&config, i)?.repository_urls;
        let _ = save_batch(&fetched);
        links_acc.append(&mut fetched);
    }

    Ok(links_acc)
}

fn get_page(config: &GithubConfig, page: i32) -> Result<CrawlResult, CrawlError> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        header::ACCEPT,
        header::HeaderValue::from_str("application/vnd.github.v3+json").unwrap(),
    );

    let url = build_url(page);

    let client = reqwest::Client::new();
    let response_result = client
        .get(&url)
        .headers(headers)
        .basic_auth(&config.user, Some(&config.token))
        .send();

    let mut response = match response_result {
        Ok(ok) => Ok(ok),
        Err(e) => {
            error!("Fetching github page {} failed due to {}", page, e);
            Err(CrawlError::FetchError)
        }
    }?;


    let json = match response.json::<serde_json::Value>() {
        Ok(ok) => Ok(ok),
        Err(e) => {
            error!(
                "Parsing response for github page {} failed due to {}",
                page, e
            );
            Err(CrawlError::ParseError("Json parsing failed".to_owned()))
        }
    }?;

    let repository_urls = parse_result(json)?;

    Ok(CrawlResult { repository_urls })
}

fn parse_result(json: Value) -> Result<Vec<String>, CrawlError> {
    let obj_array = match &json {
        Value::Array(arr) => Ok(arr.clone()),
        other => {
            error!("Couldn't parse result as array: {:?}", other);
            Err(CrawlError::ParseError("Array parsing failed".to_owned()))
        }
    }?;

    let link_results = obj_array.iter().map(|x| match &x["ssh_url"] {
        Value::String(url) => Ok(url),
        other => {
            error!("ssh_url not found, found {:?}", other);
            Err(CrawlError::ParseError("Url parsing failed".to_owned()))
        }
    });

    let links: Result<Vec<_>, _> = link_results
        .into_iter()
        .map(|r| r.map(|l| l.clone()))
        .collect();

    links
}

fn build_url(page: i32) -> String {
    API_URL.to_owned()
        + "/user/repos?"
        + "per_page="
        + &PER_PAGE.to_string()
        + "&page="
        + &page.to_string()
}
