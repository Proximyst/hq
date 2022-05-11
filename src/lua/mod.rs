use crate::prelude::*;
use mlua::{prelude::*, MultiValue, Result as LuaResult, Value};
use thiserror::Error;

pub mod enumeration;

pub fn setup_lua() -> Result<Lua, LuaError> {
    let lua = Lua::new();

    lua.globals()
        .set("enum", lua.create_function(self::enumeration::lua_enum)?)?;

    // We want a new print function, because we want it to use tracing::info!.
    lua.globals()
        .set("print", lua.create_function(tracing_info_print)?)?;

    Ok(lua)
}

#[derive(Error, Debug)]
pub enum LuaError {
    #[error("error in lua runtime: {}", .0)]
    Mlua(#[from] mlua::Error),
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
