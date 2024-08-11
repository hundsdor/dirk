use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_error::{abort, proc_macro_error};
use std::fmt::Debug;
use syn::{FnArg, ItemImpl, Pat, Type};

mod expectable;
mod syntax;
mod util;

mod scoped_inject;
mod singleton_inject;
mod static_inject;

#[derive(Debug)]
enum ParsingError {
    Wrapped(syn::parse::Error),
    InvalidItemImpl(ItemImpl),
    InvalidNumberOfFunctions(ItemImpl),
    InvalidPath,
    UnexpectedFnArg(FnArg),
    UnexpectedPat(Pat),
    UnexpectedType(Type),
}

impl ParsingError {
    fn emit(&self) -> ! {
        match self {
            ParsingError::Wrapped(_e) => abort!(Span::call_site(), self),
            ParsingError::InvalidItemImpl(item_impl) => abort!(item_impl, self),
            ParsingError::InvalidNumberOfFunctions(item_impl) => abort!(item_impl, self),
            ParsingError::UnexpectedFnArg(arg) => abort!(arg, self),
            ParsingError::UnexpectedPat(pat) => abort!(pat, self),
            ParsingError::UnexpectedType(ty) => abort!(ty, self),
            ParsingError::InvalidPath => abort!(Span::call_site(), self),
        }
    }
}

impl ToString for ParsingError {
    fn to_string(&self) -> String {
        match self {
            ParsingError::Wrapped(e) => e.to_string(),
            ParsingError::InvalidItemImpl(_) => {
                "#[*_inject] is expected to be placed on a inherent impl!".to_owned()
            }
            ParsingError::InvalidNumberOfFunctions(_) => {
                "#[*_inject] is expected to be placed on an impl with exactely one function"
                    .to_owned()
            }
            ParsingError::UnexpectedFnArg(_) => {
                "#[*_inject] is to be placed on an impl with a function having no receiver"
                    .to_owned()
            }
            ParsingError::UnexpectedPat(_) => "Found invalid kind of argument".to_owned(),
            ParsingError::UnexpectedType(_) => "Found invalid kind of type".to_owned(),
            ParsingError::InvalidPath => "Found invalid kind of path".to_owned(),
        }
    }
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn static_inject(attr: TokenStream, item: TokenStream) -> TokenStream {
    let res = static_inject::_macro(attr, item);

    match res {
        Ok(item) => item,
        Err(e) => e.emit(),
    }
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn scoped_inject(attr: TokenStream, item: TokenStream) -> TokenStream {
    let res = scoped_inject::_macro(attr, item);

    match res {
        Ok(item) => item,
        Err(e) => e.emit(),
    }
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn singleton_inject(attr: TokenStream, item: TokenStream) -> TokenStream {
    let res = singleton_inject::_macro(attr, item);

    match res {
        Ok(item) => item,
        Err(e) => e.emit(),
    }
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[cfg(test)]
mod tests {}
