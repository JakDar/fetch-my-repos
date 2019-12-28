mod integration {
    pub mod bitbucket;
    pub mod common;
    pub mod github;
    pub mod gitlab;
}

mod config;
mod io;

use quicli::prelude::*;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Cli {
    /// Prodvider: gitlab(gl) / github(gh) / bitbucket(bb)
    provider: String,
    // Quick and easy logging setup you get for free with quicli
    #[structopt(flatten)]
    verbosity: Verbosity,
}

enum Provider {
    Github,
    Gitlab,
    Bitbucket,
}

fn other_error<T>(msg: &str) -> std::io::Result<T> {
    eprintln!("{}", msg);
    Err(std::io::Error::from(std::io::ErrorKind::Other))
}

fn main() -> std::io::Result<()> {
    //TODO:Bcm - use logger and verbosity intead of println
    let args = Cli::from_args();
    let provider_string: String = args.provider;

    let provider = match provider_string.as_ref() {
        "gh" | "github" => Ok(Provider::Github),
        "bb" | "bitbucket" => Ok(Provider::Bitbucket),
        "gl" | "gitlab" => Ok(Provider::Gitlab),
        other => {
            eprintln!("Unknown provider {}", other);
            Err(std::io::Error::from(std::io::ErrorKind::Other))
        }
    }?;

    // TODO:bcm - not unwrap
    let cfg = config::load().unwrap();

    let (tmp_cache, cache) = match &provider {
        Provider::Bitbucket => (io::BITBUCKET_CACHE_TMP, io::BITBUCKET_CACHE),
        Provider::Gitlab => (io::GITLAB_CACHE_TMP, io::GITLAB_CACHE),
        Provider::Github => (io::GITHUB_CACHE_TMP, io::GITHUB_CACHE),
    };

    let save_batch: &dyn Fn(&Vec<String>) -> std::io::Result<()> = &|x| {
        io::save_lines(
            x,
            &io::filename_in_gclone_dir(tmp_cache),
            /*append:*/ true,
        )
    };

    let result = match &provider {
        Provider::Bitbucket => {
            let config = cfg
                .bitbucket
                .map(Ok)
                .unwrap_or(other_error("Bitbucket config not found"));
            integration::bitbucket::get_all(&config?, save_batch)
        }
        Provider::Gitlab => {
            let config = cfg
                .gitlab
                .map(Ok)
                .unwrap_or(other_error("Gitlab config not found"));
            integration::gitlab::get_all(&config?, save_batch)
        }
        Provider::Github => {
            let config = cfg
                .github
                .map(Ok)
                .unwrap_or(other_error("Gitlab config not found"));
            integration::github::get_all(&config?, save_batch)
        }
    };

    let _ = std::fs::create_dir(io::filename_in_gclone_dir(""));

    match result {
        Ok(res) => {
            println!("Finished caching");
            io::save_lines(&res, &io::filename_in_gclone_dir(cache), false)?;
            std::fs::remove_file(io::filename_in_gclone_dir(tmp_cache))?
        }
        Err(e) => eprintln!(
            "Saving to {} failed with {:?}",
            io::filename_in_gclone_dir(cache),
            e
        ),
    };

    Ok(())
}
