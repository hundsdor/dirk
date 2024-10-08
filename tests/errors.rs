use test_case::test_case;

mod check_output;

#[test_case("coffee", "missing_provides")]
#[test_case("coffee", "use_inject_on_impl")]
#[test_case("coffee", "component_on_impl")]
#[test_case("coffee", "component_type_mismatch")]
#[test_case("coffee", "component_type_mismatch_generics")]
#[test_case("coffee", "component_missing_dependency")]
#[test_case("coffee", "component_missing_binding")]
#[test_case("coffee", "provides_on_trait")]
#[test_case("coffee", "provides_on_empty_impl")]
#[test_case("coffee", "provides_on_impl_with_more_than_one_function")]
#[test_case("coffee", "provides_invalid_return_type")]
fn test_errors_coffee(path: &str, name: &str) {
    check_output::test_main(path, name);
}
