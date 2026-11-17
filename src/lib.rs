#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

// Disable the standard library unless the "std" feature is active
#![cfg_attr(not(feature = "std"), no_std)]

// Bring heap-allocated types (String, Vec, Box) into scope whenever
// either the "alloc" or "std" feature is active
#[cfg(feature = "alloc")]
extern crate alloc;

// (Optional) Explicitly reference the std crate when the std feature is enabled,
// although standard library items are in the prelude by default.
#[cfg(feature = "std")]
extern crate std;


// Our internal module defining private items
mod internal {

    // Error handling matters
    pub mod error;
    pub mod result;

    // I/O layer, fully hidden inside
    mod io {
        mod marker;
        pub mod reader;
        pub mod writer;
    }

    // Serde layer, with partially exposed items
    pub mod serde {
        pub mod de;
        pub mod ser;
    }

}

// RE-exports some items to the public-facing API layer

pub use internal::error::UbjError as Error;
pub use internal::result::UbjResult as Result;

#[cfg(feature = "std")]
pub use internal::serde::ser::to_writer;

#[cfg(feature = "alloc")]
pub use internal::serde::ser::to_vec;

pub use internal::serde::ser::to_slice;


#[cfg(feature = "std")]
pub use internal::serde::de::from_buf_reader;

#[cfg(feature = "alloc")]
pub use internal::serde::de::from_vec;

#[cfg(feature = "alloc")] // TODO from_slice should work in no_std strict environments too!
pub use internal::serde::de::from_slice;


