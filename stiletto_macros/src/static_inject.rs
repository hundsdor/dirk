use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_quote,
    punctuated::Punctuated,
    token::{Gt, Lt},
    AngleBracketedGenericArguments, File, GenericArgument, Item, ItemImpl, ItemStruct,
};

use crate::{
    syntax::{
        get_call_path, get_constructor_call, get_factory_ty, get_fields, get_generics,
        get_injectable, get_providers,
    },
    util::type_provider,
    ParsingError,
};

pub(crate) fn _macro(_attr: TokenStream, item: TokenStream) -> Result<TokenStream, ParsingError> {
    let input_impl = syn::parse::<ItemImpl>(item).map_err(ParsingError::Wrapped)?;

    let (ident, formal_fields, actual_fields) = get_fields(&input_impl)?;
    let (injectable_ty, injectable_path) = get_injectable(&input_impl)?;
    let impl_generics = get_generics(&input_impl)?;
    let (factory_ty, factory_path) = get_factory_ty(&injectable_ty)?;
    let (fields_providers, formal_providers, actual_providers, providers_getter) =
        get_providers(&formal_fields, true)?;

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

    let struct_factory: ItemStruct = parse_quote! {
        pub(crate) struct #factory_path #impl_generics {
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

            pub fn create(#formal_providers) -> Self {
                Self::new(#actual_providers)
            }

            fn newInstance(#formal_fields) -> #injectable_ty {
                #constructor_call
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

    Ok(TokenStream::from(expaned))
}