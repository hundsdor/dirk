use shaku::{module, Component, HasComponent};

module! {
    GenericComponent<T: Component<GenericComponent<T>>> {
        components = [T],
        providers = []
    }
}

impl Component<GenericComponent<u64>> for u64 {
    type Interface = u64;

    type Parameters = ();

    fn build(
        context: &mut shaku::ModuleBuildContext<GenericComponent<u64>>,
        params: Self::Parameters,
    ) -> Box<Self::Interface> {
        Box::new(42)
    }
}

fn main() {
    let component = GenericComponent::<u64>::builder().build();
    let answer: &u64 = component.resolve_ref();
    assert_eq!(*answer, 42);
}
