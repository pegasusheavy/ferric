//! Content Projection for Ferric Components
//!
//! Content projection allows components to accept and render child content passed to them,
//! enabling powerful component composition patterns.
//!
//! ## Basic Usage
//!
//! In your component template, use `<fe-content>` to mark where projected content should appear:
//!
//! ```html
//! <!-- card.component.html -->
//! <div class="card">
//!     <div class="card-body">
//!         <fe-content></fe-content>
//!     </div>
//! </div>
//! ```
//!
//! When using the component, any child content is projected into the slot:
//!
//! ```html
//! <app-card>
//!     <h2>Card Title</h2>
//!     <p>Card content goes here.</p>
//! </app-card>
//! ```
//!
//! ## Named Slots
//!
//! Use the `select` attribute to create named slots with CSS selectors:
//!
//! ```html
//! <!-- card.component.html -->
//! <div class="card">
//!     <div class="card-header">
//!         <fe-content select="[card-header]"></fe-content>
//!     </div>
//!     <div class="card-body">
//!         <fe-content select="[card-body]"></fe-content>
//!     </div>
//!     <div class="card-footer">
//!         <fe-content select="[card-footer]"></fe-content>
//!     </div>
//!     <!-- Default slot for unmatched content -->
//!     <fe-content></fe-content>
//! </div>
//! ```
//!
//! Usage:
//!
//! ```html
//! <app-card>
//!     <h2 card-header>Card Title</h2>
//!     <p card-body>Main content here.</p>
//!     <button card-footer>Action</button>
//!     <span>This goes to default slot</span>
//! </app-card>
//! ```
//!
//! ## Selectors
//!
//! The `select` attribute supports CSS selectors:
//! - `[attr]` - Elements with an attribute
//! - `.class` - Elements with a class
//! - `tag` - Elements by tag name
//! - `tag.class` - Combined selectors
//!
//! ## Fallback Content
//!
//! Content inside `<fe-content>` is shown when nothing is projected:
//!
//! ```html
//! <fe-content select="[header]">
//!     <h2>Default Header</h2>
//! </fe-content>
//! ```

mod projection;
mod selector;
mod slot;

pub use projection::{
    ContentProjection, ProjectedContent, ProjectionContext,
    project_content, create_projection_context,
};
pub use selector::{ContentSelector, SelectorKind, matches_selector};
pub use slot::{ContentSlot, SlotRef, find_slots, replace_slots};

