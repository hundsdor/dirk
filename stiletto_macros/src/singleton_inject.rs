use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse_quote,
    punctuated::Punctuated,
    token::{Gt, Lt, PathSep},
    AngleBracketedGenericArguments, GenericArgument, Ident, Item, ItemImpl, ItemStatic,
    ItemStruct, PathArguments, PathSegment,
};

use crate::{
    syntax::{
        get_call_path, get_constructor_call, get_factory_ty, get_fields, get_injectable,
        get_instance_name, get_providers, wrap_call, wrap_type,
    },
    util::{segments, type_arc, type_provider, type_rwlock},
    Result,
};

pub(crate) fn _macro(_attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let input_impl = syn::parse::<ItemImpl>(item)?;

    let (ident, formal_fields, actual_fields) = get_fields(&input_impl)?;
    let (injectable_ty, injectable_path) = get_injectable(&input_impl)?;
    let impl_generics = input_impl.generics.clone();
    let (factory_ty, factory_path) = get_factory_ty(&injectable_ty)?;
    let (_fields_providers, formal_providers, _actual_providers, providers_getter) =
        get_providers(&formal_fields, false)?;

    //#######
    // Wrapping type by Arc<RwLock<T>>

    let injectable_ty = wrap_type(injectable_ty, type_rwlock);
    let injectable_ty = wrap_type(injectable_ty, type_arc);

    //
    //#######

    let provider_ty = {
        let provider_generics = {
            let mut args = Punctuated::new();
            let arg = GenericArgument::Type(injectable_ty.clone());
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

    let injected = get_call_path(&injectable_path, ident);
    let constructor_call = get_constructor_call(injected, actual_fields);

    //#######
    // Wrapping constructor by Arc::new(RwLock::new(...))

    let constructor_call = wrap_call(constructor_call, segments!("std", "sync", "RwLock", "new"));
    let constructor_call = wrap_call(constructor_call, segments!("std", "sync", "Arc", "new"));

    //
    //#######

    let factory_instance_name = get_instance_name(&factory_path);

    let factory_call = get_call_path(&factory_path, Ident::new("new", Span::call_site()));
    let factory_constructor_call = get_constructor_call(factory_call, Punctuated::new());

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
                    singleton: Self::new_instance(#providers_getter),
                }
            }

            pub fn create(#formal_providers) -> Self {
                #factory_instance_name.clone()
            }

            fn new_instance(#formal_fields) -> #injectable_ty {
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

    let expaned = quote! { #(#items)* };
    Ok(TokenStream::from(expaned))
}
