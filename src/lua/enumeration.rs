use crate::prelude::*;
use mlua::prelude::*;
use serde::Serialize;
use std::collections::HashSet;

#[allow(unused_macros)]
#[macro_export]
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

pub fn lua_enum<'l>(l: &'l Lua, args: mlua::MultiValue<'l>) -> mlua::Result<mlua::Value<'l>> {
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
    trace!(?names, "lua_enum: creating enum type");

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
