use std::env::var;
use std::fs::OpenOptions;
use std::io::Write as IoWrite;

pub fn save_lines(lines: &Vec<String>, filename: &str, append: bool) -> std::io::Result<()> {
    let mut file = if append {
        OpenOptions::new()
            .append(true)
            .create(true)
            .open(filename)?
    } else {
        OpenOptions::new()
            .create(true)
            .write(true)
            .open(filename)?
    };

    for line in lines.iter() {
        if let Err(e) = writeln!(&mut file, "{}", &line) {
            eprintln!("Couldn't write to file: {}", e);
        }
    }
    Ok(())
}

pub fn filename_in_gclone_dir(filename: &str) -> String {
    format!("{}/.gclone/{}", var("HOME").unwrap(), filename)
}

pub const GITLAB_CACHE: &str = "gitlab-cache";
pub const GITLAB_CACHE_TMP: &str = "gitlab-cache-tmp";
pub const GITHUB_CACHE: &str = "github-cache";
pub const GITHUB_CACHE_TMP: &str = "github-cache-tmp";
pub const BITBUCKET_CACHE: &str = "bitbucket-cache";
pub const BITBUCKET_CACHE_TMP: &str = "bitbucket-cache-tmp";
