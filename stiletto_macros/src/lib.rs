use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use proc_macro_error::{abort, proc_macro_error};
use std::fmt::Debug;
use syn::{
    FnArg, GenericArgument, GenericParam, ItemImpl, Pat, PathArguments, ReturnType, TraitItem, Type,
};

mod expectable;
mod syntax;
mod util;

mod scoped_inject;
mod singleton_inject;
mod static_inject;

mod binding;
mod component;

#[derive(Debug)]
enum ParsingError {
    Wrapped(syn::parse::Error),
    InvalidItemImpl(ItemImpl),
    InvalidNumberOfFunctions(ItemImpl),
    InvalidNumberOfGenericArgs(Type),
    InvalidPath,
    UnexpectedFnArg(FnArg),
    UnexpectedPat(Pat),
    UnexpectedType(Type),
    UnexpectedTraitItem(TraitItem),
    UnexpectedReturnType(ReturnType),
    UnexpectedGenericArgument(GenericArgument),
    UnexpectedGenericParam(GenericParam),
    UnexpectedPathArguments(PathArguments),
    BindingNotFound(Ident),
    InvalidType(Type),
}

impl ParsingError {
    fn emit(&self) -> ! {
        match self {
            ParsingError::Wrapped(_e) => abort!(Span::call_site(), self),
            ParsingError::InvalidItemImpl(item_impl) => abort!(item_impl, self),
            ParsingError::InvalidNumberOfFunctions(item_impl) => abort!(item_impl, self),
            ParsingError::InvalidNumberOfGenericArgs(ty) => abort!(ty, self),
            ParsingError::InvalidPath => abort!(Span::call_site(), self),
            ParsingError::UnexpectedFnArg(arg) => abort!(arg, self),
            ParsingError::UnexpectedPat(pat) => abort!(pat, self),
            ParsingError::UnexpectedType(ty) => abort!(ty, self),
            ParsingError::UnexpectedTraitItem(trait_item) => abort!(trait_item, self),
            ParsingError::UnexpectedReturnType(return_type) => abort!(return_type, self),
            ParsingError::UnexpectedGenericArgument(generic_argument) => {
                abort!(generic_argument, self)
            }
            ParsingError::UnexpectedGenericParam(generic_param) => {
                abort!(generic_param, self)
            }
            ParsingError::UnexpectedPathArguments(path_arguments) => {
                abort!(path_arguments, self)
            }
            ParsingError::BindingNotFound(ident) => abort!(ident, self),
            ParsingError::InvalidType(ty) => abort!(ty, self),
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
            ParsingError::InvalidNumberOfGenericArgs(_) => {
                "Found invalid number of generic arguments".to_owned()
            }
            ParsingError::InvalidPath => "Found invalid kind of path".to_owned(),
            ParsingError::UnexpectedFnArg(_) => {
                "#[*_inject] is to be placed on an impl with a function having no receiver"
                    .to_owned()
            }
            ParsingError::UnexpectedPat(_) => "Found invalid kind of argument".to_owned(),
            ParsingError::UnexpectedType(_) => "Found invalid kind of type".to_owned(),
            ParsingError::UnexpectedTraitItem(_) => "Found invalid kind of trait item".to_owned(),
            ParsingError::UnexpectedReturnType(_) => "Found invalid kind of return type".to_owned(),
            ParsingError::UnexpectedGenericArgument(_) => {
                "Found invalid kind of generic argument".to_owned()
            }
            ParsingError::UnexpectedGenericParam(_) => {
                "Found invalid kind of generic param".to_owned()
            }
            ParsingError::UnexpectedPathArguments(_) => {
                "Found invalid kind of path arguments".to_owned()
            }
            ParsingError::BindingNotFound(ident) => {
                format!("Did not find binding {}", ident.to_string())
            }
            ParsingError::InvalidType(_) => "Found unexpected type".to_owned(),
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
pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream {
    let res = component::_macro(attr, item);

    match res {
        Ok(item) => item,
        Err(e) => e.emit(),
    }
}

#[cfg(test)]
mod tests {}
