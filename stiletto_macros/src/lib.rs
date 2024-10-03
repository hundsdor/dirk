use inject::{scoped_inject, singleton_inject, static_inject};
use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

#[macro_use]
mod errors;

mod expectable;
mod stringify;
mod syntax;
mod util;

mod component;
mod inject;
mod use_injectable;

pub(crate) const FACTORY_PREFIX_SINGLETON: &str = "SingletonFactory";
pub(crate) const FACTORY_PREFIX_SCOPED: &str = "ScopedFactory";
pub(crate) const FACTORY_PREFIX_STATIC: &str = "StaticFactory";

#[proc_macro_error]
#[proc_macro_attribute]
pub fn static_inject(attr: TokenStream, item: TokenStream) -> TokenStream {
    let res = static_inject::_macro(attr, item);

    match res {
        Ok(item) => item,
        Err(e) => e.abort(),
    }
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn scoped_inject(attr: TokenStream, item: TokenStream) -> TokenStream {
    let res = scoped_inject::_macro(attr, item);

    match res {
        Ok(item) => item,
        Err(e) => e.abort(),
    }
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn singleton_inject(attr: TokenStream, item: TokenStream) -> TokenStream {
    let res = singleton_inject::_macro(attr, item);

    match res {
        Ok(item) => item,
        Err(e) => e.abort(),
    }
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn use_injectable(attr: TokenStream, item: TokenStream) -> TokenStream {
    let res = use_injectable::_macro(attr, item);

    match res {
        Ok(item) => item,
        Err(e) => e.abort(),
    }
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream {
    let res = component::_macro(attr, item);

    match res {
        Ok(item) => item,
        Err(e) => e.abort(),
    }
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn __component(attr: TokenStream, item: TokenStream) -> TokenStream {
    let res = component::_macro_helper(attr, item);

    match res {
        Ok(item) => item,
        Err(e) => e.abort(),
    }
}

#[cfg(test)]
mod tests {}
