use proc_macro2::Ident;
use syn::{
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Comma, Paren},
    Expr, ExprCall, ExprPath, GenericArgument, GenericParam, Path, PathArguments, PathSegment,
    Type, TypePath,
};

pub(crate) fn get_call_path(ty: &TypePath, call_ident: Ident) -> syn::ExprPath {
    let mut segments = ty.path.segments.clone();

    let call_segment = PathSegment {
        ident: call_ident,
        arguments: PathArguments::None,
    };
    segments.push(call_segment);

    let path = Path {
        leading_colon: None,
        segments,
    };

    ExprPath {
        attrs: Vec::new(),
        qself: None,
        path,
    }
}

pub(crate) fn get_constructor_call(injected: ExprPath, args: Punctuated<Expr, Comma>) -> syn::Expr {
    let expr_call = ExprCall {
        attrs: Vec::new(),
        func: Box::new(Expr::Path(injected)),
        paren_token: Paren::default(),
        args,
    };

    Expr::Call(expr_call)
}

pub(crate) fn get_instance_name(base: &TypePath) -> Ident {
    let mut s = String::new();
    let segments = &base.path.segments;

    for segment in segments {
        s.push_str(&segment.ident.to_string().to_uppercase());
    }

    Ident::new(&s, base.span())
}

pub(crate) fn wrap_call(wrapped: Expr, wrapper_path: fn() -> Path) -> Expr {
    let mut args = Punctuated::new();
    args.push(wrapped);

    let path = wrapper_path();

    let expr_path = ExprPath {
        attrs: Vec::new(),
        qself: None,
        path,
    };

    let expr_call = ExprCall {
        attrs: Vec::new(),
        func: Box::new(Expr::Path(expr_path)),
        paren_token: Paren::default(),
        args,
    };

    Expr::Call(expr_call)
}

pub(crate) fn map_generic_params(
    params: Punctuated<GenericParam, Comma>,
) -> Punctuated<GenericArgument, Comma> {
    let mut ret = Punctuated::new();

    for param in params {
        let mapped = match param {
            GenericParam::Lifetime(lt_param) => GenericArgument::Lifetime(lt_param.lifetime),
            GenericParam::Type(ty_param) => {
                let path = Path::from(ty_param.ident);
                let type_path = TypePath { qself: None, path };
                let ty = Type::Path(type_path);
                GenericArgument::Type(ty)
            }
            GenericParam::Const(const_param) => {
                let path = Path::from(const_param.ident);
                let expr_path = ExprPath {
                    attrs: Vec::new(),
                    qself: None,
                    path,
                };
                let expr = Expr::Path(expr_path);
                GenericArgument::Const(expr)
            }
        };
        ret.push(mapped);
    }

    ret
}
