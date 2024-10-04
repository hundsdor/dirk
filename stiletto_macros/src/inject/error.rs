use proc_macro_error::abort;
use syn::{ItemImpl, Type};

use crate::{
    errors::ExpectableError,
    errors::{InfallibleError, SyntaxError},
    stringify::StringifyError,
};

pub(crate) type InjectResult<T> = std::result::Result<T, InjectError>;

#[derive(Debug)]
pub(crate) enum InjectError {
    Infallible(InfallibleError<InjectSyntaxError>),
    Stringify(StringifyError),
    Logic(InjectLogicError),
}

impl_abort!(InjectError);
impl_from_error!(InjectError);
impl_from_infallible_error!(InjectError, InjectSyntaxError);

#[derive(Debug)]
pub(crate) enum InjectSyntaxError {
    FailedToParseInput(syn::Error),
    ExpectedImpl(syn::Error),
}

impl SyntaxError for InjectSyntaxError {
    fn abort(self) -> ! {
        match self {
            Self::ExpectedImpl(e) => abort!(
                e.span(),
                e.to_string();
                help = "`#[*_inject]` is expected to be placed on an impl block"
            ),
            Self::FailedToParseInput(e) => abort!(e.span(), e.to_string()),
        }
    }
}

#[derive(Debug)]
pub(crate) enum InjectLogicError {
    InvalidFunctionCount(ItemImpl, usize),
    InvalidReturnType(Type),
}

impl From<InjectLogicError> for InjectError {
    fn from(value: InjectLogicError) -> Self {
        Self::Logic(value)
    }
}

impl InjectLogicError {
    fn abort(self) -> ! {
        match self {
            InjectLogicError::InvalidFunctionCount(item_impl, len) => {
                abort!(item_impl, format!("#[*_inject] is supposed to be placed on an impl block containing one single function - found {} functions instead", len))
            }
            InjectLogicError::InvalidReturnType(ty) => {
                abort!(ty, "#[*_inject] is supposed to be placed on an impl block containing a function returning `Self`")
            }
        }
    }
}
