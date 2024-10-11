use proc_macro_error::abort;
use syn::{punctuated::Punctuated, token::Comma, FnArg, ItemImpl, Type};

use crate::{
    errors::ExpectableError,
    errors::{InfallibleError, SyntaxError},
};

pub(crate) type ProvidesResult<T> = std::result::Result<T, ProvidesError>;

#[derive(Debug)]
pub(crate) enum ProvidesError {
    Infallible(InfallibleError<ProvidesSyntaxError>),
    Logic(ProvidesLogicError),
}

impl_abort!(ProvidesError);
impl_from_infallible_error!(ProvidesError, ProvidesSyntaxError);

#[derive(Debug)]
pub(crate) enum ProvidesSyntaxError {
    FailedToParseInput(syn::Error),
    ExpectedImpl(syn::Error),
}

impl SyntaxError for ProvidesSyntaxError {
    fn abort(self) -> ! {
        match self {
            Self::ExpectedImpl(e) => abort!(
                e.span(),
                e.to_string();
                help = "`#[*_provides]` is expected to be placed on an impl block"
            ),
            Self::FailedToParseInput(e) => abort!(e.span(), e.to_string()),
        }
    }
}

#[derive(Debug)]
pub(crate) enum ProvidesLogicError {
    InvalidFunctionCount(ItemImpl, usize),
    InvalidReturnType(Type),
    SingletonWithArgs(Punctuated<FnArg, Comma>),
}

impl From<ProvidesLogicError> for ProvidesError {
    fn from(value: ProvidesLogicError) -> Self {
        Self::Logic(value)
    }
}

impl ProvidesLogicError {
    fn abort(self) -> ! {
        match self {
            ProvidesLogicError::InvalidFunctionCount(item_impl, len) => {
                abort!(item_impl, format!("#[*_provides] is supposed to be placed on an impl block containing one single function - found {} functions instead", len))
            }
            ProvidesLogicError::InvalidReturnType(ty) => {
                abort!(ty, "#[*_provides] is supposed to be placed on an impl block containing a function returning `Self`")
            }
            ProvidesLogicError::SingletonWithArgs(args) => {
                abort!(
                    args,
                    "An instance provided as singleton cannot depend on any arguments."
                )
            }
        }
    }
}
