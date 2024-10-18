use test_case::test_case;

mod check_output;

#[test_case("coffee", "blueprint")]
#[test_case("coffee", "component_order_of_bindings")]
#[test_case("application", "test_generics")]
fn test_examples(path: &str, name: &str) {
    check_output::test_main(path, name);
}
