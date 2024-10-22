use proc_macro_error::abort;
use syn::UseGlob;

use crate::{
    errors::ExpectableError,
    errors::{InfallibleError, SyntaxError},
};

pub(crate) type UseComponentResult<T> = std::result::Result<T, UseComponentError>;

#[derive(Debug)]
pub(crate) enum UseComponentError {
    Infallible(InfallibleError<UseComponentSyntaxError>),
    Logic(UseComponentLogicError),
}

impl_abort!(UseComponentError);
impl_from_infallible_error!(UseComponentError, UseComponentSyntaxError);

#[derive(Debug)]
pub(crate) enum UseComponentSyntaxError {
    FailedToParseInput(syn::Error),
    ExpectedUse(syn::Error),
}

impl SyntaxError for UseComponentSyntaxError {
    fn abort(self) -> ! {
        match self {
            Self::ExpectedUse(e) => abort!(
                e.span(),
                e.to_string();
                help = "#[use_component] is expected to be placed on a use item"
            ),
            Self::FailedToParseInput(e) => abort!(e.span(), e.to_string()),
        }
    }
}

#[derive(Debug)]
pub(crate) enum UseComponentLogicError {
    FoundGlob(UseGlob),
}

impl From<UseComponentLogicError> for UseComponentError {
    fn from(value: UseComponentLogicError) -> Self {
        Self::Logic(value)
    }
}

impl UseComponentLogicError {
    fn abort(self) -> ! {
        match self {
            UseComponentLogicError::FoundGlob(use_glob) => abort!(
                use_glob,
                "#[use_component] on wildcard use items is not supported"
            ),
        }
    }
}
