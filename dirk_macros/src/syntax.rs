use proc_macro2::Span;
use syn::{
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Comma, Fn, Gt, Lt, Paren, RArrow},
    AngleBracketedGenericArguments, Block, FnArg, GenericArgument, Generics, Ident, ImplItem,
    ImplItemFn, Path, PathArguments, Signature, Type, Visibility,
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

pub(crate) fn wrap_path(wrapped: Type, getter_type: fn(PathArguments, Span) -> Path) -> Path {
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

pub(crate) fn mk_fn(
    ident: Ident,
    vis: Visibility,
    generics: Generics,
    inputs: Punctuated<FnArg, Comma>,
    return_type: Type,
    block: Block,
) -> ImplItem {
    let sig = Signature {
        constness: None,
        asyncness: None,
        unsafety: None,
        abi: None,
        fn_token: Fn::default(),
        ident,
        generics,
        paren_token: Paren::default(),
        inputs,
        variadic: None,
        output: syn::ReturnType::Type(RArrow::default(), Box::new(return_type)),
    };

    ImplItem::Fn(ImplItemFn {
        attrs: Vec::new(),
        vis,
        defaultness: None,
        sig,
        block,
    })
}
