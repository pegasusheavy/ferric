//! SSR Components for the demo application.
//!
//! This module contains various components that implement the `Renderable` trait,
//! demonstrating how to create server-rendered components with Ferric SSR.

mod counter;
mod layout;
mod pages;
mod todo;
mod user;

pub use counter::Counter;
pub use layout::Layout;
pub use pages::{AboutPage, HomePage, NotFoundPage};
pub use todo::{TodoItem, TodoList};
pub use user::UserCard;

