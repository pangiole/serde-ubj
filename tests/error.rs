use core::{fmt, error};

// A helper function that forces the compiler to check trait bounds
fn assert_trait_bounds<T: fmt::Debug + fmt::Display + error::Error>() {}

#[test]
fn serde_ubj_error_should_implement_core_traits() {
    assert_trait_bounds::<serde_ubj::Error>();
}

// TODO code more integration tests for the UbjError type