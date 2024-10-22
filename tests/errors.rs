use test_case::test_case;

mod check_output;

#[test_case("coffee", "missing_provides")]
#[test_case("coffee", "use_inject_on_impl")]
#[test_case("coffee", "component_on_impl")]
#[test_case("coffee", "component_type_mismatch")]
#[test_case("coffee", "component_type_mismatch_generics")]
#[test_case("coffee", "component_wron_binding_kind")]
#[test_case("coffee", "component_missing_dependency")]
#[test_case("coffee", "component_too_few_dependencies")]
#[test_case("coffee", "component_too_many_dependencies")]
#[test_case("coffee", "component_missing_binding")]
#[test_case("coffee", "component_cycle")]
#[test_case("coffee", "component_singleton_with_dependencies")]
#[test_case("coffee", "provides_on_trait")]
#[test_case("coffee", "provides_on_empty_impl")]
#[test_case("coffee", "provides_on_impl_with_more_than_one_function")]
#[test_case("coffee", "provides_invalid_return_type")]
#[test_case("coffee", "provides_singleton_with_args")]
#[test_case("coffee", "provides_duplicate")]
#[test_case("application", "component_binding_impl_trait")]
#[test_case("application", "component_function_returning_impl_trait")]
#[test_case("application", "component_wrapped_impl_trait")]
#[test_case("application", "component_unwrapped_impl_trait")]
#[test_case("car", "use_component_on_fn")]
fn test_errors(path: &str, name: &str) {
    check_output::test_main(path, name);
}
