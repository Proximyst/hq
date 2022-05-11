#![warn(future_incompatible)]
#![deny(rust_2018_idioms)]
#![deny(rust_2021_compatibility)]
#![deny(nonstandard_style)]
#![deny(clippy::all)]

mod lua;
pub mod prelude {
    pub use tracing::{debug, error, info, trace, warn};
}

use self::prelude::*;
use clap::Parser;
use color_eyre::{eyre::WrapErr as _, Report};
use std::path::PathBuf;
use tracing_subscriber::{prelude::*, EnvFilter};

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// The directory to find requests and scripts.
    #[clap(short, long, default_value = "./http")]
    sources: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Report> {
    color_eyre::install().wrap_err("error while setting up eyre")?;

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(EnvFilter::from_default_env())
        .try_init()
        .wrap_err("error while setting up tracing")?;

    let args = <Args as Parser>::try_parse().wrap_err("could not read command arguments")?;

    let _lua = self::lua::setup_lua(&args.sources)
        .await
        .wrap_err("error while setting up Lua")?;

    Ok(())
}
