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
use color_eyre::{eyre::WrapErr as _, Report};
use tracing_subscriber::{prelude::*, EnvFilter};

#[tokio::main]
async fn main() -> Result<(), Report> {
    color_eyre::install().wrap_err("error while setting up eyre")?;

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(EnvFilter::from_default_env())
        .try_init()
        .wrap_err("error while setting up tracing")?;

    let lua = self::lua::setup_lua().wrap_err("error while setting up Lua")?;

    lua.load(
        r#"
            local testing = enum { "A" }
            assert(testing.A)
            print(testing.A)
            print(testing.B)
        "#,
    )
    .exec_async()
    .await
    .wrap_err("error while running Lua test code")?;

    Ok(())
}
