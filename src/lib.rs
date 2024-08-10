use std::marker::PhantomData;

#[macro_export]
macro_rules! module {
    () => {};
}

#[macro_export]
macro_rules! component {
    ($name:ident, [$($module:ty),*], [$($fn_name:ident: $result:ty),*]) => {
        use stiletto_macros::factory_method;

        struct $name {}

        impl $name {
            fn build() -> Self {
                Self {}
            }
        }
    };
}

pub trait Provider<T> {
    fn get(&self) -> T;
}

#[cfg(test)]
mod tests {}
