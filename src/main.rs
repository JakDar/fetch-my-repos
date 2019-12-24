mod integration {
    mod bitbucket;
    mod github;
    pub mod gitlab;
}

mod config;
mod io;

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
        io::save_lines(
            x,
            &io::filename_in_glclone_dir(io::GITLAB_CACHE_TMP),
            /*append:*/ true,
        )
    });

    let _ = std::fs::create_dir(io::filename_in_glclone_dir(""));

    match result {
        Ok(res) => {
            println!("Finished caching");
            io::save_lines(
                &res.repository_urls,
                &io::filename_in_glclone_dir(io::GITLAB_CACHE),
                false,
            )?;
            std::fs::remove_file(io::filename_in_glclone_dir(io::GITLAB_CACHE_TMP))?
        }
        Err(e) => eprintln!(
            "Saving to {} failed with {:?}",
            io::filename_in_glclone_dir(io::GITLAB_CACHE),
            e
        ),
    };
    Ok(())
}
