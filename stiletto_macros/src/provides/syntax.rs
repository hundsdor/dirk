use itertools::Itertools;
use proc_macro2::{Ident, Span};
use syn::{
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Colon, Comma, Dot, Dyn, Gt, Lt, Paren, PathSep},
    AngleBracketedGenericArguments, Expr, ExprCall, ExprField, ExprMethodCall, ExprPath, Field,
    FieldValue, FnArg, GenericArgument, GenericParam, ImplItem, ImplItemFn, ItemImpl, Member, Path,
    PathArguments, PathSegment, TraitBound, Type, TypeParamBound, TypePath, TypeTraitObject,
};

use crate::{
    errors::InfallibleError,
    expectable::{
        FnArgExpectable, ImplItemExpectable, PatExpectable, ReturnTypeExpectable, TypeExpectable,
    },
    syntax::wrap_type,
    util::{type_provider, type_rc},
};

use super::{
    error::{ProvidesLogicError, ProvidesResult},
    ProvidesMacroInput,
};

pub(crate) fn get_first_function(input_impl: &ItemImpl) -> ProvidesResult<ImplItemFn> {
    let items = &input_impl.items;

    let first_function = items
        .iter()
        .filter(|f| matches!(f, ImplItem::Fn(_)))
        .exactly_one()
        .map_err(|e| {
            ProvidesLogicError::InvalidFunctionCount(
                input_impl.clone(),
                e.try_len()
                    .expect("Number of functions in an impl block needs to be finite"),
            )
        })?;

    let function = first_function.as_fn()?;

    Ok(function.clone())
}

pub(crate) fn get_fields(
    input_impl: &ItemImpl,
) -> ProvidesResult<(Ident, Punctuated<FnArg, Comma>, Punctuated<Expr, Comma>)> {
    let first_function = get_first_function(input_impl)?;

    let function_ident = first_function.sig.ident;

    let formal_fields: Punctuated<FnArg, Comma> = first_function.sig.inputs;

    let actual_fields = formal_fields
        .iter()
        .map(|f| {
            let pat_type = f.as_typed()?;
            let pat_ident = pat_type.pat.as_ident()?;

            let ident = pat_ident.ident.clone();

            let _member = syn::Member::Named(ident.clone());
            let expr = Expr::Path(ExprPath {
                attrs: Vec::new(),
                qself: None,
                path: Path::from(ident),
            });

            Ok(expr)
        })
        .collect::<ProvidesResult<Punctuated<Expr, Comma>>>()?;

    Ok((function_ident, formal_fields, actual_fields))
}

pub(crate) fn get_injectable(input_impl: &ItemImpl) -> ProvidesResult<(Type, TypePath)> {
    let ty = {
        let function = get_first_function(input_impl)?;
        let fun_ty = function.sig.output.as_type()?.1.as_ref().clone();

        let type_path = fun_ty.as_path()?;

        if type_path.path.is_ident("Self") {
            (*input_impl.self_ty).clone()
        } else {
            return Err(ProvidesLogicError::InvalidReturnType(fun_ty))?;
        }
    };

    let mut type_path = input_impl.self_ty.as_path()?.clone();
    let last = type_path
        .path
        .segments
        .last_mut()
        .ok_or_else(|| InfallibleError::EmptyPath(ty.span()))?;
    last.arguments = PathArguments::None;

    Ok((ty, type_path))
}

pub(crate) fn get_factory_ty(
    input: &ProvidesMacroInput,
    injectable_ty: &Type,
    params: &Punctuated<GenericParam, Comma>,
) -> ProvidesResult<(Type, TypePath)> {
    let args = map_generic_params(params.clone());
    let angle_bracketed = AngleBracketedGenericArguments {
        colon2_token: None,
        lt_token: Lt::default(),
        args,
        gt_token: Gt::default(),
    };
    let generic_args = PathArguments::AngleBracketed(angle_bracketed);

    let mut factory_ty = injectable_ty.clone();
    let span = factory_ty.span();
    let path = factory_ty.as_path_mut()?;

    let last = path
        .path
        .segments
        .last_mut()
        .ok_or_else(|| InfallibleError::EmptyPath(span))?;

    last.ident = Ident::new(
        &format!("{}{}", input.factory_prefix(), last.ident),
        last.ident.span(),
    );
    last.arguments = generic_args.clone();

    let mut factory_path = factory_ty.as_path()?.clone();
    let last = factory_path
        .path
        .segments
        .last_mut()
        .ok_or_else(|| InfallibleError::EmptyPath(span))?;
    last.arguments = PathArguments::None;

    Ok((factory_ty, factory_path))
}

pub(crate) fn get_providers(
    formal_fields: &Punctuated<FnArg, Comma>,
    add_self: bool,
) -> ProvidesResult<(
    Punctuated<FnArg, Comma>,
    Punctuated<Field, Comma>,
    Punctuated<FieldValue, Comma>,
    Punctuated<Expr, Comma>,
)> {
    let mut fn_args = Punctuated::new();
    let mut fields = Punctuated::new();
    let mut field_values = Punctuated::new();
    let mut exprs = Punctuated::new();

    for f in formal_fields {
        let mut pat_type = f.as_typed()?.clone();

        let ident = {
            let pat = &mut pat_type.pat;
            let pat_ident = pat.as_ident_mut()?;

            let ident = Ident::new(
                &format!("{}_provider", pat_ident.ident),
                pat_ident.ident.span(),
            );

            pat_ident.ident = ident.clone();

            ident
        };
        let ty = {
            let ty = &mut pat_type.ty;

            let provider_type = wrap_type((**ty).clone(), type_provider);

            let type_path = provider_type.as_path()?;

            let trait_bound = TraitBound {
                paren_token: None,
                modifier: syn::TraitBoundModifier::None,
                lifetimes: None,
                path: type_path.path.clone(),
            };
            let bound = TypeParamBound::Trait(trait_bound);

            let mut bounds = Punctuated::new();
            bounds.push(bound);

            let trait_object = TypeTraitObject {
                dyn_token: Some(Dyn::default()),
                bounds,
            };
            let dyn_type = Type::TraitObject(trait_object);

            let wrapped_ty = wrap_type(dyn_type, type_rc);

            *ty = Box::new(wrapped_ty.clone());

            wrapped_ty
        };

        let fn_arg: FnArg = FnArg::Typed(pat_type);

        let field: Field = Field {
            attrs: Vec::new(),
            vis: syn::Visibility::Inherited,
            mutability: syn::FieldMutability::None,
            ident: Some(ident.clone()),
            colon_token: Some(Colon::default()),
            ty,
        };

        let field_value: FieldValue = {
            let member = syn::Member::Named(ident.clone());
            let expr = Expr::Path(ExprPath {
                attrs: Vec::new(),
                qself: None,
                path: Path::from(ident.clone()),
            });

            FieldValue {
                attrs: Vec::new(),
                member,
                colon_token: None,
                expr,
            }
        };

        let expr = {
            let receiver = if add_self {
                let mut segments = Punctuated::new();

                let self_ident = Ident::new("self", Span::call_site());
                segments.push(PathSegment {
                    ident: self_ident,
                    arguments: PathArguments::None,
                });

                let path = Path {
                    leading_colon: None,
                    segments,
                };

                let self_expr = ExprPath {
                    attrs: Vec::new(),
                    qself: None,
                    path,
                };
                let member = Member::Named(ident);

                let expr_field = ExprField {
                    attrs: Vec::new(),
                    base: Box::new(Expr::Path(self_expr)),
                    dot_token: Dot::default(),
                    member,
                };
                Expr::Field(expr_field)
            } else {
                let segment = PathSegment {
                    ident,
                    arguments: PathArguments::None,
                };

                let mut segments = Punctuated::new();
                segments.push(segment);

                let path = Path {
                    leading_colon: None,
                    segments,
                };

                let expr_path = ExprPath {
                    attrs: Vec::new(),
                    qself: None,
                    path,
                };

                Expr::Path(expr_path)
            };
            let get_ident = Ident::new("get", Span::call_site());

            let method_call = ExprMethodCall {
                attrs: Vec::new(),
                receiver: Box::new(receiver),
                dot_token: Dot::default(),
                method: get_ident,
                turbofish: None,
                paren_token: Paren::default(),
                args: Punctuated::new(),
            };

            Expr::MethodCall(method_call)
        };

        fn_args.push(fn_arg);
        fields.push(field);
        field_values.push(field_value);
        exprs.push(expr);
    }

    Ok((fn_args, fields, field_values, exprs))
}

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

pub(crate) fn wrap_call(wrapped: Expr, wrapper_path: Punctuated<PathSegment, PathSep>) -> Expr {
    let mut args = Punctuated::new();
    args.push(wrapped);

    let path = Path {
        leading_colon: None,
        segments: wrapper_path,
    };

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

fn map_generic_params(
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
