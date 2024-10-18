use proc_macro2::Span;
use syn::{
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Gt, Lt},
    AngleBracketedGenericArguments, GenericArgument, PathArguments, Type,
};

pub(crate) fn wrap_type(wrapped: Type, getter_type: fn(PathArguments, Span) -> Type) -> Type {
    let span = wrapped.span();
    let arg = GenericArgument::Type(wrapped);

    let mut args = Punctuated::new();
    args.push(arg);

    let generic_arguments = AngleBracketedGenericArguments {
        colon2_token: None,
        lt_token: Lt::default(),
        args,
        gt_token: Gt::default(),
    };
    getter_type(PathArguments::AngleBracketed(generic_arguments), span)
}
