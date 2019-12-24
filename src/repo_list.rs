use std::fs::OpenOptions;
use std::io::Write as IoWrite;

pub fn save_lines(lines: &Vec<String>, filename: &str, append: bool) -> std::io::Result<()> {
    let mut file = if append {
        OpenOptions::new().append(true).open(filename)?
    } else {
        OpenOptions::new().write(true).open(filename)?
    };

    for line in lines.iter() {
        if let Err(e) = writeln!(&mut file, "{}", &line) {
            eprintln!("Couldn't write to file: {}", e);
        }
    }
    Ok(())
}

pub const GITLAB_CACHE: &str = "gitlab-cache";
pub const GITLAB_CACHE_TMP: &str = "gitlab-cache-tmp";
// const BITBUCKET_CACHE: &str = "bitbucket-cache";
// const BITBUCKET_CACHE_TMP: &str = "bitbucket-cache-tmp";
