//! # Ferric Animations
//!
//! Angular-inspired animation system for Ferric framework.
//!
//! ## Features
//!
//! - **Triggers**: Named animation triggers with states
//! - **States**: Define animation states with styles
//! - **Transitions**: Animate between states with timing functions
//! - **Keyframes**: Complex multi-step animations
//! - **Stagger**: Animate lists with delays
//! - **Route Animations**: Transitions between route changes
//!
//! ## Example
//!
//! ```rust,no_run
//! use ferric_animations::*;
//!
//! // Define a fade animation trigger
//! let fade = trigger("fade", vec![
//!     state("void", style(vec![
//!         ("opacity", "0"),
//!     ])),
//!     state("*", style(vec![
//!         ("opacity", "1"),
//!     ])),
//!     transition("void => *", animate("300ms ease-in")),
//!     transition("* => void", animate("300ms ease-out")),
//! ]);
//!
//! // Apply to component
//! // #[component(animations = [fade])]
//! ```

pub mod animation;
pub mod builder;
pub mod keyframes;
pub mod route;
pub mod stagger;
pub mod state;
pub mod timing;
pub mod transition;
pub mod trigger;

pub use animation::*;
pub use builder::*;
pub use keyframes::*;
pub use route::*;
pub use stagger::*;
pub use state::*;
pub use timing::*;
pub use transition::*;
pub use trigger::*;

/// Re-export for convenience
pub mod prelude {
    pub use crate::{
        animate, animation, keyframes, route_animation, stagger, state, style, transition,
        trigger, AnimationBuilder, AnimationMetadata, AnimationPlayer, AnimationState,
        AnimationTrigger, EasingFunction, Keyframe, RouteAnimation, StaggerConfig,
        TimingFunction, Transition, TransitionMatcher,
    };
}

