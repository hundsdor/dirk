use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse_quote,
    punctuated::Punctuated,
    token::{Gt, Lt, PathSep},
    AngleBracketedGenericArguments, GenericArgument, Ident, Item, ItemImpl, ItemStruct,
    PathArguments, PathSegment,
};

use crate::{
    syntax::wrap_type,
    util::{segments, type_provider, type_rc, type_refcell},
};

use super::{
    error::{InjectResult, InjectSyntaxError},
    syntax::{
        get_call_path, get_constructor_call, get_factory_ty, get_fields, get_injectable,
        get_providers, wrap_call,
    },
};

pub(crate) fn _macro(_attr: TokenStream, item: TokenStream) -> InjectResult<TokenStream> {
    let input = super::InjectMacroInput::Scoped;
    let input_impl = syn::parse::<ItemImpl>(item).map_err(InjectSyntaxError::ExpectedImpl)?;

    let (ident, formal_fields, actual_fields) = get_fields(&input_impl)?;
    let (injectable_ty, injectable_path) = get_injectable(&input_impl)?;
    let impl_generics = input_impl.generics.clone();
    let (factory_ty, factory_path) = get_factory_ty(&input, &injectable_ty, &impl_generics.params)?;
    let (_fields_providers, formal_providers, actual_providers, providers_getter) =
        get_providers(&formal_fields, false)?;

    //#######
    // Wrapping type by Rc<RefCell<T>>

    let injectable_ty = wrap_type(injectable_ty, type_refcell);
    let injectable_ty = wrap_type(injectable_ty, type_rc);

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
    // Wrapping constrcutor by Rc::new(RefCell::new(...))

    let constructor_call = wrap_call(constructor_call, segments!("std", "cell", "RefCell", "new"));
    let constructor_call = wrap_call(constructor_call, segments!("std", "rc", "Rc", "new"));

    //
    //#######

    let struct_factory: ItemStruct = parse_quote! {
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
                Self::new(#actual_providers)
            }

            fn new_instance(#formal_fields) -> #injectable_ty {
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

    let expaned = quote! { #(#items)* };
    Ok(TokenStream::from(expaned))
}
