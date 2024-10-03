

use crate::{FACTORY_PREFIX_SCOPED, FACTORY_PREFIX_SINGLETON, FACTORY_PREFIX_STATIC};

pub(crate) mod scoped_inject;
pub(crate) mod singleton_inject;
pub(crate) mod static_inject;

mod error;
mod syntax;

#[derive(Debug)]
pub enum InjectMacroInput {
    Static,
    Scoped,
    Singleton,
}

impl InjectMacroInput {
    pub(crate) fn factory_prefix(&self) -> &'static str {
        match self {
            InjectMacroInput::Static => FACTORY_PREFIX_STATIC,
            InjectMacroInput::Scoped => FACTORY_PREFIX_SCOPED,
            InjectMacroInput::Singleton => FACTORY_PREFIX_SINGLETON,
        }
    }
}
