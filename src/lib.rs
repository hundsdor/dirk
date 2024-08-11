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

#[cfg(test)]
mod tests {}
