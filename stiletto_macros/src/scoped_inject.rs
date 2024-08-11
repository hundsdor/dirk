use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse_quote,
    punctuated::Punctuated,
    token::{Gt, Lt, Paren, PathSep},
    AngleBracketedGenericArguments, Expr, ExprCall, ExprPath, File, GenericArgument, Ident, Item,
    ItemImpl, ItemStruct, Path, PathArguments, PathSegment, Type, TypePath,
};

use crate::{
    syntax::{
        get_call_path, get_constructor_call, get_factory_ty, get_fields, get_generics,
        get_injectable, get_providers,
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
    // Wrapping type by Rc<RefCell<T>>

    let ref_cell = segments!("std", "cell", "RefCell");
    let injectable_ty = wrap_injectable(injectable_ty, ref_cell).unwrap();

    let rc = segments!("std", "rc", "Rc");
    let injectable_ty = wrap_injectable(injectable_ty, rc).unwrap();

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
    // Wrapping constrcutor by Rc::new(RefCell::new(...))

    let ref_cell_new = segments!("std", "cell", "RefCell", "new");
    let constructor_call = wrap_call(constructor_call, ref_cell_new).unwrap();

    let rc_new = segments!("std", "rc", "Rc", "new");
    let constructor_call = wrap_call(constructor_call, rc_new).unwrap();

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
                    singleton: Self::newInstance(#providers_getter),
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
