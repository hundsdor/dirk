use syn::{
    punctuated::Punctuated,
    token::{Gt, Lt},
    AngleBracketedGenericArguments, GenericArgument, Type,
};

pub(crate) fn wrap_type(
    wrapped: Type,
    getter_type: fn(AngleBracketedGenericArguments) -> Type,
) -> Type {
    let arg = GenericArgument::Type(wrapped);

    let mut args = Punctuated::new();
    args.push(arg);

    let generic_arguments = AngleBracketedGenericArguments {
        colon2_token: None,
        lt_token: Lt::default(),
        args,
        gt_token: Gt::default(),
    };
    getter_type(generic_arguments)
}
