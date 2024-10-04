use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

#[macro_use]
mod errors;

mod expectable;
mod syntax;
mod util;

mod component;
mod provides;
mod use_injectable;

pub(crate) const FACTORY_PREFIX_SINGLETON: &str = "SingletonFactory";
pub(crate) const FACTORY_PREFIX_SCOPED: &str = "ScopedFactory";
pub(crate) const FACTORY_PREFIX_STATIC: &str = "StaticFactory";

#[proc_macro_error]
#[proc_macro_attribute]
pub fn provides(attr: TokenStream, item: TokenStream) -> TokenStream {
    let res = provides::_macro(attr, item);

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
