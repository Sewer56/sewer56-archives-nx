#[cfg(not(feature = "nightly"))]
pub use allocator_api2::alloc::*;
#[cfg(feature = "nightly")]
pub use std::alloc::*;

#[cfg(not(feature = "nightly"))]
pub use allocator_api2::boxed::*;
#[cfg(feature = "nightly")]
pub use alloc::boxed::*;

#[cfg(not(feature = "nightly"))]
pub use allocator_api2::collections::*;
#[cfg(feature = "nightly")]
pub use alloc::collections::*;

#[cfg(not(feature = "nightly"))]
pub use allocator_api2::vec;
#[cfg(feature = "nightly")]
pub use alloc::vec;

#[cfg(not(feature = "nightly"))]
pub use allocator_api2::vec::*;
#[cfg(feature = "nightly")]
pub use alloc::vec::*;
