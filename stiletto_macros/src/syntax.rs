use proc_macro2::{Ident, Span};
use syn::{
    punctuated::Punctuated,
    token::{Colon, Comma, Dot, Dyn, Gt, Lt, Paren},
    AngleBracketedGenericArguments, Expr, ExprCall, ExprField, ExprMethodCall, ExprPath, Field,
    FieldValue, FnArg, GenericArgument, Generics, ImplItem, ImplItemFn, ItemImpl, Member, Path,
    PathArguments, PathSegment, TraitBound, Type, TypeParamBound, TypePath, TypeTraitObject,
};

use crate::{
    expectable::{FnArgExpectable, PatExpectable, TypeExpectable},
    util::{type_provider, type_rc},
    ParsingError,
};

pub(crate) fn get_first_function(input_impl: &ItemImpl) -> Result<ImplItemFn, ParsingError> {
    if input_impl.trait_.is_some() {
        return Err(ParsingError::InvalidItemImpl(input_impl.clone()));
    }

    let items = &input_impl.items;
    let mut functions = items.iter().filter(|f| matches!(f, ImplItem::Fn(_)));

    let first_function = functions
        .next()
        .ok_or_else(|| ParsingError::InvalidNumberOfFunctions(input_impl.clone()))?;

    if functions.next().is_some() {
        return Err(ParsingError::InvalidNumberOfFunctions(input_impl.clone()));
    }

    let ImplItem::Fn(function) = first_function else {
        unreachable!()
    };

    Ok(function.clone())
}

pub(crate) fn get_fields(
    input_impl: &ItemImpl,
) -> Result<(Ident, Punctuated<FnArg, Comma>, Punctuated<Expr, Comma>), ParsingError> {
    let first_function = get_first_function(input_impl)?;

    let function_ident = first_function.sig.ident;

    let formal_fields: Punctuated<FnArg, Comma> = first_function.sig.inputs;

    let actual_fields = formal_fields
        .iter()
        .map(|f| {
            let pat_type = f.as_typed()?;
            let pat_ident = pat_type.pat.as_ident()?;

            let ident = pat_ident.ident.clone();

            let member = syn::Member::Named(ident.clone());
            let expr = syn::Expr::Path(ExprPath {
                attrs: Vec::new(),
                qself: None,
                path: Path::from(ident),
            });

            Ok(expr)
        })
        .collect::<Result<Punctuated<Expr, Comma>, ParsingError>>()?;

    Ok((function_ident, formal_fields, actual_fields))
}

pub(crate) fn get_injectable(input_impl: &ItemImpl) -> Result<(Box<Type>, TypePath), ParsingError> {
    let ty = input_impl.self_ty.clone();

    let mut path = ty.as_path()?.clone();
    let last = path
        .path
        .segments
        .last_mut()
        .ok_or(ParsingError::InvalidPath)?;
    last.arguments = PathArguments::None;

    Ok((ty, path))
}

pub(crate) fn get_generics(input_impl: &ItemImpl) -> Result<Generics, ParsingError> {
    let generics = input_impl.generics.clone();
    Ok(generics)
}

pub(crate) fn get_factory_ty(injectable_ty: &Type) -> Result<(Type, TypePath), ParsingError> {
    let mut factory_ty = injectable_ty.clone();
    let path = factory_ty.as_path_mut()?;

    let last = path
        .path
        .segments
        .last_mut()
        .ok_or(ParsingError::InvalidPath)?;
    last.ident = Ident::new(&format!("Factory{}", last.ident), last.ident.span());

    let mut factory_path = factory_ty.as_path()?.clone();
    let last = factory_path
        .path
        .segments
        .last_mut()
        .ok_or(ParsingError::InvalidPath)?;
    last.arguments = PathArguments::None;

    Ok((factory_ty, factory_path))
}

pub(crate) fn get_providers(
    formal_fields: &Punctuated<FnArg, Comma>,
    add_self: bool,
) -> Result<
    (
        Punctuated<FnArg, Comma>,
        Punctuated<Field, Comma>,
        Punctuated<FieldValue, Comma>,
        Punctuated<Expr, Comma>,
    ),
    ParsingError,
> {
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
                &format!("{}Provider", pat_ident.ident),
                pat_ident.ident.span(),
            );

            pat_ident.ident = ident.clone();

            ident
        };
        let ty = {
            let ty = &mut pat_type.ty;

            let generic_arg = GenericArgument::Type(*ty.clone());

            let mut generic_args = Punctuated::new();
            generic_args.push(generic_arg);

            let generics = AngleBracketedGenericArguments {
                colon2_token: None,
                lt_token: Lt::default(),
                args: generic_args,
                gt_token: Gt::default(),
            };
            let provider_type = type_provider(generics);

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

            let arg = GenericArgument::Type(dyn_type);

            let mut args = Punctuated::new();
            args.push(arg);

            let generic_arguments = AngleBracketedGenericArguments {
                colon2_token: None,
                lt_token: Lt::default(),
                args,
                gt_token: Gt::default(),
            };
            let wrapped_ty = type_rc(generic_arguments);

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
            let expr = syn::Expr::Path(ExprPath {
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

pub(crate) fn get_call_path(ty: &TypePath, call_ident: Ident) -> Result<ExprPath, ParsingError> {
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

    let expr_path = ExprPath {
        attrs: Vec::new(),
        qself: None,
        path,
    };

    Ok(expr_path)
}

pub(crate) fn get_constructor_call(
    injected: ExprPath,
    args: Punctuated<Expr, Comma>,
) -> Result<Expr, ParsingError> {
    let expr_call = ExprCall {
        attrs: Vec::new(),
        func: Box::new(Expr::Path(injected)),
        paren_token: Paren::default(),
        args,
    };

    Ok(Expr::Call(expr_call))
}

pub(crate) fn get_instance_name(base: &TypePath) -> Ident {
    let mut s = String::new();
    let segments = &base.path.segments;

    for segment in segments {
        s.push_str(&segment.ident.to_string());
    }

    Ident::new(&s, Span::call_site())
}
