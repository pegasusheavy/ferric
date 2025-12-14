//! Pipes for transforming displayed values in templates.
//!
//! Pipes are a way to transform data for display in templates without
//! modifying the underlying data.
//!
//! ## Usage in Templates
//!
//! ```html
//! <!-- Basic usage -->
//! <p>{{ name | uppercase }}</p>
//! <p>{{ price | currency:'USD' }}</p>
//! <p>{{ birthday | date:'short' }}</p>
//!
//! <!-- Chaining pipes -->
//! <p>{{ name | lowercase | slice:0:10 }}</p>
//!
//! <!-- With arguments -->
//! <p>{{ amount | number:'1.2-2' }}</p>
//! ```
//!
//! ## Built-in Pipes
//!
//! ### Text Pipes
//! - `uppercase` - Convert to uppercase
//! - `lowercase` - Convert to lowercase
//! - `titlecase` - Convert to title case
//! - `trim` - Remove whitespace
//!
//! ### Number Pipes
//! - `number` - Format numbers with locale
//! - `currency` - Format as currency
//! - `percent` - Format as percentage
//!
//! ### Date Pipes
//! - `date` - Format dates
//!
//! ### Utility Pipes
//! - `json` - Convert to JSON
//! - `slice` - Extract substring/subarray
//! - `default` - Provide default value
//!
//! ### Async Pipes
//! - `async` - Subscribe to promises/async values
//!
//! ## Custom Pipes
//!
//! ```rust
//! use ferric_core::pipes::{Pipe, PipeArgs};
//!
//! struct ReversePipe;
//!
//! impl Pipe for ReversePipe {
//!     fn name(&self) -> &'static str { "reverse" }
//!
//!     fn transform(&self, value: &str, _args: &PipeArgs) -> String {
//!         value.chars().rev().collect()
//!     }
//! }
//! ```

mod async_pipe;
mod builtin;
mod di;
mod registry;
mod transform;

pub use async_pipe::{
    AsyncPipe, AsyncState, AsyncValue, AsyncPipeRegistry,
    async_pending, async_resolved, async_error,
};
pub use builtin::*;
pub use di::{PipesModule, InjectablePipeRegistry, PIPES as DI_PIPES};
pub use registry::{PipeRegistry, register_pipe, get_pipe, register_builtin_pipes};
pub use transform::{
    Pipe, PipeArgs, PipeValue, ParsedPipe,
    apply_pipe, apply_pipes, parse_pipe_expression, parse_piped_expression,
};
