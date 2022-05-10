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
use color_eyre::Report;
use mlua::prelude::*;
use tracing_subscriber::{prelude::*, EnvFilter};

#[tokio::main]
async fn main() -> Result<(), Report> {
    color_eyre::install()?;

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(EnvFilter::from_default_env())
        .try_init()?;

    let lua = Lua::new();
    // Lua setup
    lua.globals().set("enum", lua.create_function(self::lua::enumeration::lua_enum)?)?;

    // We want a new print function, because we want it to use tracing::info!.
    lua.globals()
        .set("print", lua.create_function(tracing_info_print)?)?;

    lua.load(
        r#"
            local testing = enum { "A" }
            assert(testing.A)
            print(testing.A)
            print(testing.B)
        "#,
    )
    .exec_async()
    .await?;

    Ok(())
}

pub fn tracing_info_print(_: &Lua, args: mlua::MultiValue<'_>) -> mlua::Result<()> {
    // If the args are empty, nothing is allocated in practice. So let's be wasteful.
    let mut message = String::new();
    for arg in args {
        if !message.is_empty() {
            message.push('\t');
        }
        let formatted = match &arg {
            mlua::Value::Function(f) => format!("{:?}", f),
            otherwise => ron::to_string(otherwise).to_lua_err()?,
        };
        message.push_str(&formatted);
    }
    info!("lua: {}", message);
    Ok(())
}
