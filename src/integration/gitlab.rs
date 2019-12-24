use crate::config::GitlabConfig;
use reqwest;
use serde_json::Value;

const API_URL: &str = "https://gitlab.com/api/v4";
const PER_PAGE: i32 = 4;

#[derive(Debug)]
pub struct CrawlResult {
    pub total_pages: i32,
    pub total: i32,
    pub repository_urls: Vec<String>,
}

// REVIEW make save_lines a callback / function
pub fn get_all(
    config: GitlabConfig,
    save_batch: &dyn Fn(&Vec<String>) -> std::io::Result<()>,
) -> Result<CrawlResult, CrawlError> {
    let mut links_acc: Vec<String> = vec![];

    let mut first = get_page(&config, 1)?;

    let _ = save_batch(&first.repository_urls);

    links_acc.append(&mut first.repository_urls);

    for page in 2..first.total_pages + 1 {
        let mut crawled_page = get_page(&config, page)?;
        let _ = save_batch(&crawled_page.repository_urls);

        println!("{} out of {}", page, first.total_pages); // FIXME: - use verbosity
        links_acc.append(&mut crawled_page.repository_urls);
    }

    Ok(CrawlResult {
        repository_urls: links_acc,
        total: first.total,
        total_pages: first.total_pages,
    })
}

fn get_page(config: &GitlabConfig, page: i32) -> Result<CrawlResult, CrawlError> {
    let url = build_url(&config.token, page);

    let mut ala: reqwest::Response = reqwest::get(&url).unwrap();

    let json = ala.json::<serde_json::Value>().unwrap();

    let repository_urls = parse_result(json)?;

    let total_pages = extract_i32_header(ala.headers(), "x-total-pages").unwrap_or(0);
    let total = extract_i32_header(ala.headers(), "x-total").unwrap_or(0);

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

#[derive(Debug)]
pub enum CrawlError {
    ParseError,
}

fn parse_result(json: Value) -> Result<Vec<String>, CrawlError> {
    let obj_array = match &json {
        Value::Array(arr) => Ok(arr.clone()),
        _ => Err(CrawlError::ParseError),
    }?;

    let link_results = obj_array.iter().map(|x| match &x["ssh_url_to_repo"] {
        Value::String(url) => Ok(url),
        _ => Err(CrawlError::ParseError),
    });

    // FIXME: is there Result.sequence?
    let mut links: Vec<String> = vec![];

    for res in link_results {
        links.push(res?.clone());
    }
    Ok(links)
}

fn build_url(token: &str, page: i32) -> String {
    // TODO:bcm page
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
