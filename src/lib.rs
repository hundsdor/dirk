use once_cell::sync::Lazy;
use std::ops::Deref;

pub trait Provider<T> {
    fn get(&self) -> T;
}

pub struct ClonedInstanceFactory<T: Clone> {
    inner: T,
}

impl<T: Clone> ClonedInstanceFactory<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl<T: Clone> Provider<T> for ClonedInstanceFactory<T> {
    fn get(&self) -> T {
        self.inner.clone()
    }
}

pub struct ScopedInstanceFactory<T> {
    inner: std::rc::Rc<std::cell::RefCell<T>>,
}

impl<T> ScopedInstanceFactory<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner: std::rc::Rc::new(std::cell::RefCell::new(inner)),
        }
    }
}

impl<T> Provider<std::rc::Rc<std::cell::RefCell<T>>> for ScopedInstanceFactory<T> {
    fn get(&self) -> std::rc::Rc<std::cell::RefCell<T>> {
        self.inner.clone()
    }
}

pub struct Unset;
pub struct Set<T>(pub T);

pub trait InputStatus {}
impl InputStatus for Unset {}
impl<T> InputStatus for Set<T> {}

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

pub trait DirkComponent<B> {
    fn builder() -> B;
}

#[cfg(test)]
mod tests {}
