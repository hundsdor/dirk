use proc_macro2::Span;
use proc_macro_error::{abort, abort_call_site};
use std::fmt::Debug;

macro_rules! impl_abort {
    ($t:ident) => {
        impl $t {
            pub(crate) fn abort(self) -> ! {
                match self {
                    $t::Infallible(e) => e.abort(),
                    $t::Logic(u) => u.abort(),
                }
            }
        }
    };
}

macro_rules! impl_from_infallible_error {
    ($t:ident, $e:ident) => {
        impl From<$e> for InfallibleError<$e> {
            fn from(value: $e) -> Self {
                Self::Parsing(value)
            }
        }

        impl From<InfallibleError<$e>> for $t {
            fn from(value: InfallibleError<$e>) -> Self {
                Self::Infallible(value)
            }
        }

        impl From<$e> for $t {
            fn from(value: $e) -> Self {
                Into::<InfallibleError<$e>>::into(value).into()
            }
        }

        impl<T: ExpectableError + 'static> From<T> for $t {
            fn from(value: T) -> Self {
                Into::<InfallibleError<$e>>::into(value).into()
            }
        }
    };
}

pub(crate) trait ExpectableError: Debug {}

pub(crate) type InfallibleResult<T, P> = Result<T, InfallibleError<P>>;

pub(crate) trait SyntaxError {
    fn abort(self) -> !;
}

#[derive(Debug)]
pub(crate) enum InfallibleError<P: SyntaxError> {
    Parsing(P),
    UnexpectedToken(Box<dyn ExpectableError>),
    EmptyPath(Span),
}

impl<P: SyntaxError> InfallibleError<P> {
    pub(crate) fn abort(self) -> ! {
        match self {
            Self::Parsing(e) => e.abort(),
            Self::UnexpectedToken(t) => abort_call_site!(format!("{t:?}")),
            Self::EmptyPath(span) => abort!(span, "Expected non-empty path"),
        }
    }
}

impl<T: ExpectableError + 'static, P: SyntaxError> From<T> for InfallibleError<P> {
    fn from(value: T) -> Self {
        Self::UnexpectedToken(Box::new(value))
    }
}
