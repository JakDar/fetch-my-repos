use crate::config::GitlabConfig;
use crate::integration::common::*;
use quicli::prelude::*;
use reqwest;
use serde_json::Value;

const API_URL: &str = "https://gitlab.com/api/v4";
const PER_PAGE: i32 = 100;

#[derive(Debug)]
pub struct CrawlResult {
    pub total_pages: i32,
    pub total: i32,
    pub repository_urls: Vec<String>,
}

pub fn get_all(
    config: &GitlabConfig,
    save_batch: &dyn Fn(&Vec<String>) -> std::io::Result<()>,
) -> Result<Vec<String>, CrawlError> {
    let mut links_acc: Vec<String> = vec![];

    let mut first = get_page(&config, 1)?;
    info!("Processed page 1/{}", first.total_pages);

    let _ = save_batch(&first.repository_urls);

    links_acc.append(&mut first.repository_urls);

    for page in 2..first.total_pages + 1 {
        let mut crawled_page = get_page(&config, page)?;
        let _ = save_batch(&crawled_page.repository_urls);

        info!("Processed page {}/{}", page, first.total_pages);
        links_acc.append(&mut crawled_page.repository_urls);
    }

    Ok(links_acc)
}

fn get_page(config: &GitlabConfig, page: i32) -> Result<CrawlResult, CrawlError> {
    let url = build_url(&config.token, page);

    let mut response: reqwest::Response = match reqwest::get(&url) {
        Ok(res) => Ok(res),
        Err(e) => {
            error!("Fetching gitlab page {} failed due to {}", page, e);
            Err(CrawlError::FetchError)
        }
    }?;

    let json = match response.json::<serde_json::Value>() {
        Ok(res) => Ok(res),
        Err(e) => {
            error!("Parsing gitlab page {} failed due to {}", page, e);
            Err(CrawlError::ParseError("Response parsing failed".to_owned()))
        }
    }?;

    let repository_urls = parse_result(json)?;

    let total_pages = extract_i32_header(response.headers(), "x-total-pages").unwrap_or(0);
    let total = extract_i32_header(response.headers(), "x-total").unwrap_or(0);

    Ok(CrawlResult {
        repository_urls,
        total_pages,
        total,
    })
}

fn extract_i32_header(headers: &reqwest::header::HeaderMap, name: &str) -> Option<i32> {
    let header = headers.get(name)?;
    let str_value = header.to_str().ok()?;
    str_value.parse::<i32>().ok()
}

fn parse_result(json: Value) -> Result<Vec<String>, CrawlError> {
    let obj_array = match &json {
        Value::Array(arr) => Ok(arr.clone()),
        _ => {
            error!("Gitlab responed with {:?} instead of json array", json);
            Err(CrawlError::ParseError(
                "Parsing gitlab response to json failed".to_owned(),
            ))
        }
    }?;

    let link_results = obj_array.iter().map(|x| match &x["ssh_url_to_repo"] {
        Value::String(url) => Ok(url),
        _ => Err(CrawlError::ParseError(
            "Getting ssh url from gitlab response failed".to_owned(),
        )),
    });

    let links: Result<Vec<_>, _> = link_results
        .into_iter()
        .map(|r| r.map(|l| l.clone()))
        .collect();

    links
}

fn build_url(token: &str, page: i32) -> String {
    API_URL.to_owned()
        + "/projects/?"
        + "private_token="
        + &token
        + "&membership=true&simple=true"
        + "&per_page="
        + &PER_PAGE.to_string()
        + "&page="
        + &page.to_string()
}
