use proc_macro_error::abort;
use syn::UseGlob;

use crate::{
    errors::ExpectableError,
    errors::{InfallibleError, SyntaxError},
};

pub(crate) type UseInjectableResult<T> = std::result::Result<T, UseInjectableError>;

#[derive(Debug)]
pub(crate) enum UseInjectableError {
    Infallible(InfallibleError<UseInjectableSyntaxError>),
    Logic(UseInjectableLogicError),
}

impl_abort!(UseInjectableError);
impl_from_infallible_error!(UseInjectableError, UseInjectableSyntaxError);

#[derive(Debug)]
pub(crate) enum UseInjectableSyntaxError {
    FailedToParseInput(syn::Error),
    ExpectedUse(syn::Error),
}

impl SyntaxError for UseInjectableSyntaxError {
    fn abort(self) -> ! {
        match self {
            Self::ExpectedUse(e) => abort!(
                e.span(),
                e.to_string();
                help = "#[use_inject(...)] is expected to be placed on a use item"
            ),
            Self::FailedToParseInput(e) => abort!(e.span(), e.to_string()),
        }
    }
}

#[derive(Debug)]
pub(crate) enum UseInjectableLogicError {
    FoundGlob(UseGlob),
}

impl From<UseInjectableLogicError> for UseInjectableError {
    fn from(value: UseInjectableLogicError) -> Self {
        Self::Logic(value)
    }
}

impl UseInjectableLogicError {
    fn abort(self) -> ! {
        match self {
            UseInjectableLogicError::FoundGlob(use_glob) => abort!(
                use_glob,
                "#[use_provides] on wildcard use items is not supported"
            ),
        }
    }
}
