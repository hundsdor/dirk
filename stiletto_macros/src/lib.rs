use proc_macro::TokenStream;
use std::fmt::Debug;

mod expectable;
mod static_inject;

mod util {
    use proc_macro2::Span;
    use syn::{
        punctuated::Punctuated, token::PathSep, AngleBracketedGenericArguments, Ident, Path,
        PathArguments, PathSegment, Type, TypePath,
    };

    macro_rules! mk_type {
        ($ty:ident $($segments:literal)+) => {
            mk_type!{$ty [] $($segments)+}
        };

        ($ty:ident [$($segments:literal)*] $head:literal $($tail:literal)+) => {
            mk_type!{$ty [$($segments)* $head] $($tail)+}
        };

        ($ty:ident [$($segments:literal)*] $head:literal) => {

    pub(crate) fn $ty(generics: AngleBracketedGenericArguments) -> Type {
        let path =  {
            let mut segments: Punctuated<PathSegment, PathSep> = Punctuated::new();

            $(
            let segment =  {
                let ident = Ident::new($segments, Span::call_site());
                let arguments = PathArguments::None;
                PathSegment { ident, arguments}
            };
            segments.push(segment);
            )*

            let segment =  {
                let ident = Ident::new($head, Span::call_site());
                let arguments = PathArguments::AngleBracketed(generics);
                PathSegment { ident, arguments}
            };
            segments.push(segment);

            TypePath { qself: None, path:  Path { leading_colon: None, segments }}
        };
        Type::Path(path)
    }
        };
    }

    mk_type!(type_provider "stiletto" "Provider");
    mk_type!(type_rc "std" "rc" "Rc");
    mk_type!(type_arc "std" "sync" "Arc");
    mk_type!(type_rwlock "std" "sync" "RwLock");
}

enum ParsingError {
    InvalidImplType,
    InvalidNumberOfFunctions,
    InvalidReceiverType,
    InvalidFnArgType,
    InvalidTypeKind,
    InvalidPath,
    InvalidFieldType,
}

impl Debug for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParsingError::InvalidImplType => {
                f.write_str("#[*_inject] is expected to be placed on a inherent impl!")
            }
            ParsingError::InvalidNumberOfFunctions => f.write_str(
                "#[*_inject] is expected to be placed on an impl with exactely one function",
            ),
            ParsingError::InvalidReceiverType => f.write_str(
                "#[*_inject] is to be placed on an impl with a function having no receiver",
            ),
            ParsingError::InvalidFnArgType => f.write_str("Found invalid kind of argument"),
            ParsingError::InvalidTypeKind => f.write_str("Found invalid kind of type"),
            ParsingError::InvalidPath => f.write_str("Found invalid kind of path"),
            ParsingError::InvalidFieldType => f.write_str("Found invalid kind of field"),
        }
    }
}

#[proc_macro_attribute]
pub fn static_inject(attr: TokenStream, item: TokenStream) -> TokenStream {
    static_inject::_macro(attr, item)
}

#[proc_macro_attribute]
pub fn scoped_inject(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn singleton_inject(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[cfg(test)]
mod tests {}
