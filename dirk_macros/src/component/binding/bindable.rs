use proc_macro2::Ident;
use syn::{
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Comma, Paren},
    Expr, ExprCall, ExprPath, Path, PathArguments, PathSegment, Type,
};

use crate::{
    component::error::ComponentResult, errors::InfallibleError, expectable::TypeExpectable,
};

pub(crate) trait Bindable {
    fn ty(&self) -> ComponentResult<Type>;
    fn wrapped_ty(&self) -> ComponentResult<Type>;
    fn unwrap_ty<'o>(&self, other: &'o Type) -> ComponentResult<&'o Type>;
    fn hint(&self) -> &'static str;
}

pub(crate) trait FactoryBindable: Bindable {
    fn get_factory_create_call(&self) -> ComponentResult<ExprCall> {
        let path = {
            let ty = self.ty()?;

            let mut segments = ty.as_path()?.path.segments.clone();
            let last = segments
                .last_mut()
                .ok_or_else(|| InfallibleError::EmptyPath(ty.span()))?;
            last.ident = Ident::new(
                &format!("{}{}", self.factory_prefix(), last.ident),
                last.ident.span(),
            );
            last.arguments = PathArguments::None;

            let create = PathSegment {
                ident: Ident::new("create", ty.span()),
                arguments: PathArguments::None,
            };
            segments.push(create);

            Path {
                leading_colon: None,
                segments,
            }
        };
        let expr_path = ExprPath {
            attrs: Vec::new(),
            qself: None,
            path,
        };
        let fun = syn::Expr::Path(expr_path);

        Ok(ExprCall {
            attrs: Vec::new(),
            func: Box::new(fun),
            paren_token: Paren::default(),
            args: self.provider_calls(),
        })
    }

    fn provider_calls(&self) -> Punctuated<Expr, Comma>;
    fn factory_prefix(&self) -> &'static str;
}
