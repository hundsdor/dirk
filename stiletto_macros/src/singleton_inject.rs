use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse_quote,
    punctuated::Punctuated,
    token::{Gt, Lt, Paren, PathSep},
    AngleBracketedGenericArguments, Expr, ExprCall, ExprPath, File, GenericArgument, Ident, Item,
    ItemImpl, ItemStatic, ItemStruct, Path, PathArguments, PathSegment, Type, TypePath,
};

use crate::{
    syntax::{
        get_call_path, get_constructor_call, get_factory_ty, get_fields, get_generics,
        get_injectable, get_instance_name, get_providers,
    },
    util::{segments, type_provider},
    ParsingError,
};

pub(crate) fn _macro(_attr: TokenStream, item: TokenStream) -> Result<TokenStream, ParsingError> {
    let input_impl = syn::parse::<ItemImpl>(item).map_err(ParsingError::Wrapped)?;

    let (ident, formal_fields, actual_fields) = get_fields(&input_impl)?;
    let (injectable_ty, injectable_path) = get_injectable(&input_impl)?;
    let impl_generics = get_generics(&input_impl)?;
    let (factory_ty, factory_path) = get_factory_ty(&injectable_ty)?;
    let (_fields_providers, formal_providers, actual_providers, providers_getter) =
        get_providers(&formal_fields, false)?;

    //#######
    // Wrapping type by Arc<RwLock<T>>

    let rw_lock = segments!("std", "sync", "RwLock");
    let injectable_ty = wrap_injectable(injectable_ty, rw_lock).unwrap();

    let arc = segments!("std", "sync", "Arc");
    let injectable_ty = wrap_injectable(injectable_ty, arc).unwrap();

    //
    //#######

    let provider_ty = {
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
        type_provider(provider_generics)
    };

    let injected = get_call_path(&injectable_path, ident)?;
    let constructor_call = get_constructor_call(injected, actual_fields)?;

    //#######
    // Wrapping constrcutor by Arc::new(RwLock::new(...))

    let rw_lock_new = segments!("std", "sync", "RwLock", "new");
    let constructor_call = wrap_call(constructor_call, rw_lock_new).unwrap();

    let arc_new = segments!("std", "sync", "Arc", "new");
    let constructor_call = wrap_call(constructor_call, arc_new).unwrap();

    //
    //#######

    let factory_instance_name = get_instance_name(&factory_path);

    let factory_call = get_call_path(&factory_path, Ident::new("new", Span::call_site()))?;
    let factory_constructor_call = get_constructor_call(factory_call, Punctuated::new())?;

    let struct_factory: ItemStruct = parse_quote! {
        #[derive(Clone)]
        pub(crate) struct #factory_path #impl_generics {
            singleton: #injectable_ty
        }
    };

    let impl_provider_for_factory: ItemImpl = parse_quote! {

       impl #impl_generics #provider_ty for #factory_ty {
            fn get(&self) -> #injectable_ty {
                self.singleton.clone()
            }
       }
    };

    let impl_factory: ItemImpl = parse_quote! {

        impl #impl_generics #factory_ty {
            fn new(#formal_providers) -> Self {
                Self {
                    singleton: Self::newInstance(#providers_getter),
                }
            }

            pub fn create(#formal_providers) -> Self {
                #factory_instance_name.clone()
            }

            fn newInstance(#formal_fields) -> #injectable_ty {
                #constructor_call
            }
        }
    };

    let static_factory_instance: ItemStatic = parse_quote! {
        static #factory_instance_name: stiletto::FactoryInstance<#factory_path> =
            stiletto::FactoryInstance::new(|| #factory_constructor_call);
    };

    let items = vec![
        Item::Struct(struct_factory),
        Item::Impl(impl_provider_for_factory),
        Item::Impl(impl_factory),
        Item::Static(static_factory_instance),
        Item::Impl(input_impl),
    ];

    let file = File {
        shebang: None,
        attrs: Vec::new(),
        items,
    };

    let expaned = quote! { #file};

    Ok(TokenStream::from(expaned))
}

fn wrap_injectable(
    injectable_ty: Box<Type>,
    wrapper_path: Punctuated<PathSegment, PathSep>,
) -> Result<Box<Type>, ()> {
    let arg = GenericArgument::Type(*injectable_ty);

    let mut args = Punctuated::new();
    args.push(arg);

    let generic_args = AngleBracketedGenericArguments {
        colon2_token: None,
        lt_token: Lt::default(),
        args,
        gt_token: Gt::default(),
    };

    let mut segments = wrapper_path;
    let last = segments.last_mut().ok_or(())?;
    last.arguments = PathArguments::AngleBracketed(generic_args);

    let path = Path {
        leading_colon: None,
        segments,
    };
    let type_path = TypePath { qself: None, path };
    let ty = Type::Path(type_path);

    Ok(Box::new(ty))
}

fn wrap_call(call: Expr, wrapper_path: Punctuated<PathSegment, PathSep>) -> Result<Expr, ()> {
    let mut args = Punctuated::new();
    args.push(call);

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

    Ok(Expr::Call(expr_call))
}
