#![allow(internal_features)]
#![feature(rustc_attrs)]

use once_cell::sync::Lazy;
use std::ops::Deref;

pub trait Provider<T> {
    fn get(&self) -> T;
}

pub struct FactoryInstance<T>(Lazy<T>);

impl<T> FactoryInstance<T> {
    pub const fn new(constructor: fn() -> T) -> Self {
        Self(Lazy::new(constructor))
    }
}

impl<T> Deref for FactoryInstance<T> {
    type Target = Lazy<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[rustc_on_unimplemented(
    message = "unable to create a builder from `{Self}` due to errors in the injection mechanism",
    label = "injection error",
    note = "there might be an error in the `#[component(...)]` macro on the trait corresponding to `{Self}`"
)]
pub trait DirkComponent<B> {
    fn builder() -> B;
}

#[cfg(test)]
mod tests {}
