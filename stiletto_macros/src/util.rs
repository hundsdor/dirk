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

macro_rules! segments {
    ($($segments:literal),*) => {
        segments!([] $($segments)*)
    };

    ([$($segments:literal)*] $head:literal $($tail:literal)*) => {
        segments!([$($segments)* $head] $($tail)*)
    };

    ([$($segments:literal)*]) => {
        {
            let mut segments: Punctuated<PathSegment, PathSep> = Punctuated::new();

            $(
            let segment =  {
                let ident = Ident::new($segments, Span::call_site());
                let arguments = PathArguments::None;
                PathSegment { ident, arguments}
            };
            segments.push(segment);
            )*

            segments
        }
    };
}
pub(crate) use segments;
