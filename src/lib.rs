#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

// Apply the no_std attribute to this crate, which prevents it from linking to the standard library,
// under the condition that the "std" feature is not enabled.
//
// It is useful to make this create as a "hybrid crate" that can function in both standard (std)
// and bare-metal (no_std) environments.
//
#![cfg_attr(not(feature = "std"), no_std)]


#[cfg(not(any(feature = "std", feature = "alloc")))]
compile_error! {
    "serde_ubj requires that either `std` (default) or `alloc` features to be enabled"
}

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod prelude;

/// About errors
mod err;
pub use err::UbjError;

mod markers;

mod ser;
pub use ser::to_writer;

// TODO mod de
// TODO pub de::from_reader

