#![warn(future_incompatible)]
#![deny(rust_2018_idioms)]
#![deny(rust_2021_compatibility)]
#![deny(nonstandard_style)]
#![deny(clippy::all)]

pub mod prelude {
    pub use tracing::{debug, error, info, trace, warn};
}

use self::prelude::*;
use color_eyre::Report;
use mlua::prelude::*;
use serde::Serialize;
use std::collections::HashSet;
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
    lua.globals().set("enum", lua.create_function(lua_enum)?)?;

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

#[allow(unused_macros)]
macro_rules! lua_enum {
    {
        $(#[$typmeta:meta])*
        $vis:vis enum $typename:ident {
            $(
                $(#[$variantmeta:meta])*
                $variantname:ident = $luaname:literal
            ),*
            $(,)*
        }
    } => {
        $(#[$typmeta])*
        $vis enum $typename {
            $(
                $(#[$variantmeta])*
                $variantname
            ),*
        }

        impl<'l> mlua::ToLua<'l> for $typename {
            fn to_lua(self, lua: &'l Lua) -> mlua::Result<mlua::Value<'l>> {
                match self {
                    $(
                        Self::$variantname => Ok(mlua::Value::String(lua.create_string($luaname)?))
                    ),*
                }
            }
        }

        #[allow(deadcode)]
        impl $typename {
            pub fn lua_definition() -> LuaEnum {
                let mut set = HashSet::with_capacity(
                    0
                    $(+ if $luaname == $luaname { 1 } else { 1 })*
                );
                $(set.insert(String::from($luaname));)*
                LuaEnum(set)
            }
        }
    };
}

fn tracing_info_print(_: &Lua, args: mlua::MultiValue<'_>) -> mlua::Result<()> {
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

fn lua_enum<'l>(l: &'l Lua, args: mlua::MultiValue<'l>) -> mlua::Result<mlua::Value<'l>> {
    if args.len() != 1 {
        return Err("expected 1 argument, which should be a set").to_lua_err();
    }
    let set = match args.into_iter().next() {
        Some(mlua::Value::Table(t)) => t,
        Some(otherwise) => {
            return Err(format!("argument must be a set; found: {:?}", otherwise)).to_lua_err()
        }
        None => return Err("expected 1 argument").to_lua_err(),
    };

    let mut names = HashSet::new();
    for value in set.sequence_values::<String>() {
        names.insert(value?);
    }

    Ok(mlua::Value::UserData(
        l.create_ser_userdata(LuaEnum(names))?,
    ))
}

#[derive(Serialize, Debug)]
pub struct LuaEnum(HashSet<String>);

impl mlua::UserData for LuaEnum {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method("__index", |_, this, args: mlua::MultiValue<'_>| {
            let name = match args.into_iter().next() {
                Some(mlua::Value::String(name)) => name,
                _ => return Err("invalid args: must be exactly 1 string argument").to_lua_err(),
            };

            if this.0.contains(name.to_str()?) {
                return Ok(mlua::Value::String(name));
            }

            Err(format!("invalid enum value: {}", name.to_str()?)).to_lua_err()
        });
    }
}
