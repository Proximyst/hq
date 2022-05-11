use crate::prelude::*;
use mlua::{prelude::*, MultiValue, Result as LuaResult, Value};
use std::path::Path;
use thiserror::Error;
use tokio::{fs, io};

pub mod enumeration;

pub async fn setup_lua(sources: &Path) -> Result<Lua, LuaError> {
    let lua = Lua::new();

    lua.globals()
        .set("enum", lua.create_function(self::enumeration::lua_enum)?)?;

    // We want a new print function, because we want it to use tracing::info!.
    lua.globals()
        .set("print", lua.create_function(tracing_info_print)?)?;

    load_prelude(sources, &lua).await?;

    Ok(lua)
}

#[derive(Error, Debug)]
pub enum LuaError {
    #[error("error in lua runtime: {}", .0)]
    Mlua(#[from] mlua::Error),

    #[error("i/o error: {}", .0)]
    Io(#[from] io::Error),
}

async fn load_prelude(sources: &Path, lua: &Lua) -> Result<(), LuaError> {
    let file = sources.join("prelude.lua");
    if file.is_file() {
        trace!(?file, "loading prelude from file");
        let contents = fs::read_to_string(file).await?;
        lua.load(&contents).exec_async().await?;
    } else {
        trace!(?file, "skipping loading prelude from file");
    }

    let dir_path = sources.join("prelude");
    match fs::read_dir(&dir_path).await {
        Ok(mut dir) => {
            while let Some(entry) = dir.next_entry().await? {
                let name = entry.file_name();
                let name = match name.to_str() {
                    Some(n) => n,
                    None => {
                        info!(name = ?entry.file_name(), "skipping prelude file, because name was invalid UTF-8");
                        continue;
                    }
                };
                if !name.ends_with(".lua") {
                    debug!(?name, "skipping non-lua file");
                    continue;
                }

                let name = dir_path.join(name);
                trace!(?name, "reading prelude lua file");
                let contents = fs::read_to_string(name).await?;
                trace!(?contents, "loading prelude from file");
                lua.load(&contents).exec_async().await?;
            }
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            trace!(?e, "skipping loading prelude from dir");
        }
        Err(e) => Err(e)?,
    }

    Ok(())
}

fn tracing_info_print(_: &Lua, args: MultiValue<'_>) -> LuaResult<()> {
    // If the args are empty, nothing is allocated in practice. So let's be wasteful.
    let mut message = String::new();
    for arg in args {
        if !message.is_empty() {
            message.push('\t');
        }
        let formatted = match &arg {
            Value::Function(f) => format!("{:?}", f),
            otherwise => ron::to_string(otherwise).to_lua_err()?,
        };
        message.push_str(&formatted);
    }
    info!("lua: {}", message);
    Ok(())
}
