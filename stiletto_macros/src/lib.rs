use proc_macro::TokenStream;
use proc_macro2::{Ident};
use proc_macro_error::{abort_call_site, proc_macro_error};
use std::fmt::Debug;
use syn::{
    punctuated::Punctuated, token::PathSep, ItemImpl,
    PathArguments, PathSegment, Type,
};

mod expectable;
mod syntax;
mod util;

mod scoped_inject;
mod singleton_inject;
mod static_inject;

mod binding;
mod component;

pub(crate) type Result<T> = std::result::Result<T, StilettoError>;

#[derive(Debug)]
enum StilettoError {
    Parsing(syn::parse::Error),
    UnexpectedToken(Box<dyn ExpectableError>),

    Component(ComponentLogicError),
    Inject(InjectLogicError),
}

impl StilettoError {
    fn emit(&self) -> ! {
        abort_call_site!(self)
    }
}

impl ToString for StilettoError {
    fn to_string(&self) -> String {
        match self {
            StilettoError::Parsing(e) => format!("Error during parsing: {}", e.to_string()),
            StilettoError::UnexpectedToken(e) => {
                format!("Found unexpected token: {:?}", e)
            }
            StilettoError::Component(e) => {
                format!("Found logic error in #[component(...)]: {:?}", e)
            }
            StilettoError::Inject(e) => {
                format!("Found logic error in #[..._inject]: {:?}", e)
            }
        }
    }
}

trait ExpectableError: Debug {}

impl From<syn::parse::Error> for StilettoError {
    fn from(value: syn::Error) -> Self {
        Self::Parsing(value)
    }
}

impl<T: ExpectableError + 'static> From<T> for StilettoError {
    fn from(value: T) -> Self {
        Self::UnexpectedToken(Box::new(value))
    }
}

#[derive(Debug)]
enum ComponentLogicError {
    NotFound(Ident),
    InvalidGenericArgCount(Type),
    EmptyPath,
    TypeMismatch {
        fun_type: Punctuated<PathSegment, PathSep>,
        binding_type: Punctuated<PathSegment, PathSep>,
    },
    InvalidType(Type),
    InvalidPathArguments(PathArguments),
}

impl From<ComponentLogicError> for StilettoError {
    fn from(value: ComponentLogicError) -> Self {
        Self::Component(value)
    }
}

#[derive(Debug)]
enum InjectLogicError {
    InjectOnTrait(ItemImpl),
    InvalidFunctionCount(ItemImpl),
    EmptyPath,
}

impl From<InjectLogicError> for StilettoError {
    fn from(value: InjectLogicError) -> Self {
        Self::Inject(value)
    }
}

// impl ToString for ParsingError {
//     fn to_string(&self) -> String {
//         match self {
//             ParsingError::Wrapped(e) => e.to_string(),
//             ParsingError::InvalidItemImpl(_) => {
//                 "#[*_inject] is expected to be placed on a inherent impl!".to_owned()
//             }
//             ParsingError::InvalidNumberOfFunctions(_) => {
//                 "#[*_inject] is expected to be placed on an impl with exactely one function"
//                     .to_owned()
//             }
//             ParsingError::InvalidNumberOfGenericArgs(_) => {
//                 "Found invalid number of generic arguments".to_owned()
//             }
//             ParsingError::InvalidPath => "Found invalid kind of path".to_owned(),
//             ParsingError::UnexpectedFnArg(_) => {
//                 "#[*_inject] is to be placed on an impl with a function having no receiver"
//                     .to_owned()
//             }
//             ParsingError::UnexpectedPat(_) => "Found invalid kind of argument".to_owned(),
//             ParsingError::UnexpectedType(_) => "Found invalid kind of type".to_owned(),
//             ParsingError::UnexpectedTraitItem(_) => "Found invalid kind of trait item".to_owned(),
//             ParsingError::UnexpectedReturnType(_) => "Found invalid kind of return type".to_owned(),
//             ParsingError::UnexpectedGenericArgument(_) => {
//                 "Found invalid kind of generic argument".to_owned()
//             }
//             ParsingError::UnexpectedGenericParam(_) => {
//                 "Found invalid kind of generic param".to_owned()
//             }
//             ParsingError::UnexpectedPathArguments(_) => {
//                 "Found invalid kind of path arguments".to_owned()
//             }
//             ParsingError::BindingNotFound(ident) => {
//                 format!("Did not find binding {ident}")
//             }
//             ParsingError::BindingWrongType {
//                 fun_type,
//                 binding_type,
//             } => {
//                 format!(
//                     "Type of binding\n{binding_type:?}\n did not match type of function\n{fun_type:?}"
//                 )
//             }
//             ParsingError::InvalidType(_) => "Found unexpected type".to_owned(),
//         }
//     }
// }

#[proc_macro_error]
#[proc_macro_attribute]
pub fn static_inject(attr: TokenStream, item: TokenStream) -> TokenStream {
    let res = static_inject::_macro(attr, item);

    match res {
        Ok(item) => item,
        Err(e) => e.emit(),
    }
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn scoped_inject(attr: TokenStream, item: TokenStream) -> TokenStream {
    let res = scoped_inject::_macro(attr, item);

    match res {
        Ok(item) => item,
        Err(e) => e.emit(),
    }
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn singleton_inject(attr: TokenStream, item: TokenStream) -> TokenStream {
    let res = singleton_inject::_macro(attr, item);

    match res {
        Ok(item) => item,
        Err(e) => e.emit(),
    }
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream {
    let res = component::_macro(attr, item);

    match res {
        Ok(item) => item,
        Err(e) => e.emit(),
    }
}

#[cfg(test)]
mod tests {}
