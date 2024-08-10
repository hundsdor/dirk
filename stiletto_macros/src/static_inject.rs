use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
    token::{Colon, Comma, Dot, Dyn, Gt, Lt, Paren},
    AngleBracketedGenericArguments, Expr, ExprField, ExprMethodCall, ExprPath, Field, FieldValue,
    File, FnArg, GenericArgument, Generics, ImplItem, ImplItemFn, Item, ItemImpl, ItemStruct,
    Member, Path, PathArguments, PathSegment, TraitBound, Type, TypeParamBound, TypePath,
    TypeTraitObject,
};

use crate::{
    expectable::{FnArgExpectable, PatExpectable, TypeExpectable},
    util::{type_provider, type_rc},
    ParsingError,
};

pub(crate) fn _macro(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_impl = parse_macro_input!(item as ItemImpl);

    let (formal_fields, actual_fields) = get_fields(&input_impl).unwrap();
    let (injectable_ty, injectable_path) = get_injectable(&input_impl).unwrap();
    let impl_generics = get_generics(&input_impl).unwrap();
    let (factory_ty, factory_path) = get_factory_ty(&injectable_ty).unwrap();
    let (fields_providers, formal_providers, actual_providers, providers_getter) =
        get_providers(&formal_fields).unwrap();

    let provider_generics = {
        let mut args = Punctuated::new();
        let arg = GenericArgument::Type(*injectable_ty.clone());
        args.push(arg);

        AngleBracketedGenericArguments {
            colon2_token: None,
            lt_token: Lt::default(),
            args,
            gt_token: Gt::default(),
        }
    };
    let provider_ty = type_provider(provider_generics);

    let struct_factory: ItemStruct = parse_quote! {
        struct #factory_path #impl_generics {
            #fields_providers
        }
    };

    let impl_provider_for_factory: ItemImpl = parse_quote! {

       impl #impl_generics #provider_ty for #factory_ty {
            fn get(&self) -> #injectable_ty {
                Self::newInstance(#providers_getter)
            }
       }
    };

    let impl_factory: ItemImpl = parse_quote! {

        impl #impl_generics #factory_ty {
            fn new(#formal_providers) -> Self {
                Self {
                    #actual_providers
                }
            }

            fn create(#formal_providers) -> Self {
                Self::new(#actual_providers)
            }

            fn newInstance(#formal_fields) -> #injectable_ty {
                #injectable_path::new(#actual_fields)
            }
        }
    };

    let items = vec![
        Item::Struct(struct_factory),
        Item::Impl(impl_provider_for_factory),
        Item::Impl(impl_factory),
        Item::Impl(input_impl),
    ];

    let file = File {
        shebang: None,
        attrs: Vec::new(),
        items,
    };

    let expaned = quote! { #file};

    TokenStream::from(expaned)
}

pub(crate) fn get_first_function(input_impl: &ItemImpl) -> Result<ImplItemFn, ParsingError> {
    if input_impl.trait_.is_some() {
        return Err(ParsingError::InvalidImplType);
    }

    let items = &input_impl.items;
    let mut functions = items.iter().filter(|f| matches!(f, ImplItem::Fn(_)));

    let first_function = functions
        .next()
        .ok_or(ParsingError::InvalidNumberOfFunctions)?;

    if functions.next().is_some() {
        return Err(ParsingError::InvalidNumberOfFunctions);
    }

    let ImplItem::Fn(function) = first_function else {
        unreachable!()
    };

    Ok(function.clone())
}

pub(crate) fn get_fields(
    input_impl: &ItemImpl,
) -> Result<(Punctuated<FnArg, Comma>, Punctuated<FieldValue, Comma>), ParsingError> {
    let first_function = get_first_function(input_impl)?;

    let formal_fields: Punctuated<FnArg, Comma> = first_function.sig.inputs;

    let actual_fields = formal_fields
        .iter()
        .map(|f| {
            let pat_type = f.as_typed().ok_or(ParsingError::InvalidReceiverType)?;
            let pat_ident = pat_type
                .pat
                .as_ident()
                .ok_or(ParsingError::InvalidFnArgType)?;

            let ident = pat_ident.ident.clone();

            let member = syn::Member::Named(ident.clone());
            let expr = syn::Expr::Path(ExprPath {
                attrs: Vec::new(),
                qself: None,
                path: Path::from(ident),
            });

            let value = FieldValue {
                attrs: Vec::new(),
                member,
                colon_token: None,
                expr,
            };

            Ok(value)
        })
        .collect::<Result<Punctuated<FieldValue, Comma>, ParsingError>>()?;

    Ok((formal_fields, actual_fields))
}

pub(crate) fn get_injectable(input_impl: &ItemImpl) -> Result<(Box<Type>, TypePath), ParsingError> {
    let ty = input_impl.self_ty.clone();

    let mut path = ty.as_path().ok_or(ParsingError::InvalidTypeKind)?.clone();
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
    let path = factory_ty
        .as_path_mut()
        .ok_or(ParsingError::InvalidTypeKind)?;

    let last = path
        .path
        .segments
        .last_mut()
        .ok_or(ParsingError::InvalidPath)?;
    last.ident = Ident::new(&format!("Factory{}", last.ident), last.ident.span());

    let mut factory_path = factory_ty
        .as_path()
        .ok_or(ParsingError::InvalidTypeKind)?
        .clone();
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

    for f in formal_fields.iter() {
        let mut pat_type = f
            .as_typed()
            .ok_or(ParsingError::InvalidReceiverType)?
            .clone();

        let ident = {
            let pat = &mut pat_type.pat;
            let pat_ident = &mut pat.as_ident_mut().ok_or(ParsingError::InvalidFieldType)?;

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

            let type_path = provider_type
                .as_path()
                .ok_or(ParsingError::InvalidTypeKind)?;

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

            let expr_path = ExprField {
                attrs: Vec::new(),
                base: Box::new(Expr::Path(self_expr)),
                dot_token: Dot::default(),
                member,
            };
            let receiver = Expr::Field(expr_path);
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
