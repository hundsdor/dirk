//! An example of injecting generic types

use dirk_framework::component;
use dirk_framework::component::{builder::Builder, Component};

#[component(answer: cloned_instance_bind(T))]
trait GenericComponent<T: Clone + 'static> {
    fn answer(&self) -> T;
}

fn main() {
    let component = DirkGenericComponent::builder().answer(42).build();
    assert_eq!(component.answer(), 42);
}
