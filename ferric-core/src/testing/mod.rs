//! Testing utilities for Ferric applications.
//!
//! This module provides a comprehensive testing framework for Ferric components,
//! services, and other framework features.
//!
//! ## Features
//!
//! - **TestBed** - Configure and create test modules
//! - **Component Fixtures** - Render components in isolation
//! - **Mock Services** - DI overrides for testing
//! - **Async Testing** - Utilities for testing async operations
//!
//! ## Quick Start
//!
//! ```ignore
//! use ferric_core::testing::*;
//!
//! #[test]
//! fn test_my_component() {
//!     // Configure test module
//!     let test_bed = TestBed::configure()
//!         .component::<MyComponent>()
//!         .provide::<dyn MyService>(MockMyService::new())
//!         .compile();
//!
//!     // Create component fixture
//!     let fixture = test_bed.create_component::<MyComponent>();
//!
//!     // Interact with component
//!     fixture.set_input("name", "Test");
//!     fixture.detect_changes();
//!
//!     // Assert on rendered output
//!     assert!(fixture.query(".title").text().contains("Test"));
//! }
//! ```

mod test_bed;
mod fixture;
mod mock;
mod async_utils;
mod assertions;
mod dom;

pub use test_bed::*;
pub use fixture::*;
pub use mock::*;
pub use async_utils::*;
pub use assertions::*;
pub use dom::*;
