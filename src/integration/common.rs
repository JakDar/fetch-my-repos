#[derive(Debug)]
pub enum CrawlError {
    ParseError(String),
    FetchError,
}
