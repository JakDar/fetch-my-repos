mod integration {
    mod bitbucket;
    mod github;
    pub mod gitlab;
}

mod config;
mod repo_list;

use quicli::prelude::*;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Cli {
    // REVIEW - maybe use clap with enums?
    /// Prodvider: gitlab(gl) / github(gh) / bitbucket(bb)
    provider: String,
    // Quick and easy logging setup you get for free with quicli
    #[structopt(flatten)]
    verbosity: Verbosity,
}

fn main() -> std::io::Result<()> {
    // let args = Cli::from_args();
    // println!("{}", args.provider);
    let cfg = config::load().unwrap();

    let result = integration::gitlab::get_all(cfg.gitlab.unwrap(), &|x| {
        repo_list::save_lines(x, repo_list::GITLAB_CACHE_TMP, /*append:*/ true)
    });

    match result {
        Ok(res) =>{
            repo_list::save_lines(&res.repository_urls, repo_list::GITLAB_CACHE, false)?,
            println!("Finished caching")
        }
        Err(e) => eprintln!("Saving to {} failed with {:?}", repo_list::GITLAB_CACHE, e),
    };
    panic!()
}
