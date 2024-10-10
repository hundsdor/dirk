use component::{error::ComponentError, processor::ComponentMacroData};
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
    let data = ComponentMacroData::new(attr, item);
    let res = data
        .is_helper()
        .map_err(Into::<ComponentError>::into)
        .and_then(|is_helper| {
            if !is_helper {
                component::_macro(data).map_err(Into::<ComponentError>::into)
            } else {
                component::_macro_helper(data)
            }
        });

    match res {
        Ok(item) => item,
        Err(e) => e.abort(),
    }
}

#[cfg(test)]
mod tests {}
