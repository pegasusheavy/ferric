//! Lifecycle state tracking for components.

use std::fmt;

/// The current lifecycle state of a component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(Default)]
pub enum LifecycleState {
    /// Component has been created but not initialized.
    #[default]
    Created,
    /// Component is currently initializing.
    Initializing,
    /// OnInit hook has been called.
    Initialized,
    /// View is being created.
    ViewInitializing,
    /// AfterViewInit hook has been called.
    ViewInitialized,
    /// Content is being projected.
    ContentInitializing,
    /// AfterContentInit hook has been called.
    ContentInitialized,
    /// Component is fully ready and active.
    Active,
    /// Component is being checked for changes.
    Checking,
    /// Component is being destroyed.
    Destroying,
    /// Component has been destroyed.
    Destroyed,
    /// Component encountered an error.
    Error,
}

impl LifecycleState {
    /// Check if the component is in an active state (not destroyed or error).
    pub fn is_active(&self) -> bool {
        !matches!(self, LifecycleState::Destroyed | LifecycleState::Error)
    }

    /// Check if the component has completed initialization.
    pub fn is_initialized(&self) -> bool {
        matches!(
            self,
            LifecycleState::Initialized
                | LifecycleState::ViewInitializing
                | LifecycleState::ViewInitialized
                | LifecycleState::ContentInitializing
                | LifecycleState::ContentInitialized
                | LifecycleState::Active
                | LifecycleState::Checking
        )
    }

    /// Check if the component's view is ready.
    pub fn is_view_ready(&self) -> bool {
        matches!(
            self,
            LifecycleState::ViewInitialized
                | LifecycleState::ContentInitializing
                | LifecycleState::ContentInitialized
                | LifecycleState::Active
                | LifecycleState::Checking
        )
    }

    /// Check if the component is fully ready.
    pub fn is_ready(&self) -> bool {
        matches!(self, LifecycleState::Active | LifecycleState::Checking)
    }

    /// Get the next valid state in the initialization sequence.
    pub fn next_init_state(&self) -> Option<LifecycleState> {
        match self {
            LifecycleState::Created => Some(LifecycleState::Initializing),
            LifecycleState::Initializing => Some(LifecycleState::Initialized),
            LifecycleState::Initialized => Some(LifecycleState::ViewInitializing),
            LifecycleState::ViewInitializing => Some(LifecycleState::ViewInitialized),
            LifecycleState::ViewInitialized => Some(LifecycleState::ContentInitializing),
            LifecycleState::ContentInitializing => Some(LifecycleState::ContentInitialized),
            LifecycleState::ContentInitialized => Some(LifecycleState::Active),
            _ => None,
        }
    }

    /// Check if transitioning to the given state is valid.
    pub fn can_transition_to(&self, target: LifecycleState) -> bool {
        match (self, target) {
            // Normal initialization flow
            (LifecycleState::Created, LifecycleState::Initializing) => true,
            (LifecycleState::Initializing, LifecycleState::Initialized) => true,
            (LifecycleState::Initialized, LifecycleState::ViewInitializing) => true,
            (LifecycleState::ViewInitializing, LifecycleState::ViewInitialized) => true,
            (LifecycleState::ViewInitialized, LifecycleState::ContentInitializing) => true,
            (LifecycleState::ContentInitializing, LifecycleState::ContentInitialized) => true,
            (LifecycleState::ContentInitialized, LifecycleState::Active) => true,

            // Change detection cycle
            (LifecycleState::Active, LifecycleState::Checking) => true,
            (LifecycleState::Checking, LifecycleState::Active) => true,

            // Destruction from any active state
            (state, LifecycleState::Destroying) if state.is_active() => true,
            (LifecycleState::Destroying, LifecycleState::Destroyed) => true,

            // Error from any state except Destroyed
            (state, LifecycleState::Error) if *state != LifecycleState::Destroyed => true,

            _ => false,
        }
    }
}


impl fmt::Display for LifecycleState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LifecycleState::Created => write!(f, "Created"),
            LifecycleState::Initializing => write!(f, "Initializing"),
            LifecycleState::Initialized => write!(f, "Initialized"),
            LifecycleState::ViewInitializing => write!(f, "ViewInitializing"),
            LifecycleState::ViewInitialized => write!(f, "ViewInitialized"),
            LifecycleState::ContentInitializing => write!(f, "ContentInitializing"),
            LifecycleState::ContentInitialized => write!(f, "ContentInitialized"),
            LifecycleState::Active => write!(f, "Active"),
            LifecycleState::Checking => write!(f, "Checking"),
            LifecycleState::Destroying => write!(f, "Destroying"),
            LifecycleState::Destroyed => write!(f, "Destroyed"),
            LifecycleState::Error => write!(f, "Error"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialization_flow() {
        let mut state = LifecycleState::Created;

        assert!(state.can_transition_to(LifecycleState::Initializing));
        state = LifecycleState::Initializing;

        assert!(state.can_transition_to(LifecycleState::Initialized));
        state = LifecycleState::Initialized;

        assert!(state.can_transition_to(LifecycleState::ViewInitializing));
        state = LifecycleState::ViewInitializing;

        assert!(state.can_transition_to(LifecycleState::ViewInitialized));
        state = LifecycleState::ViewInitialized;

        assert!(state.can_transition_to(LifecycleState::ContentInitializing));
        state = LifecycleState::ContentInitializing;

        assert!(state.can_transition_to(LifecycleState::ContentInitialized));
        state = LifecycleState::ContentInitialized;

        assert!(state.can_transition_to(LifecycleState::Active));
    }

    #[test]
    fn test_destruction_from_active() {
        let state = LifecycleState::Active;
        assert!(state.can_transition_to(LifecycleState::Destroying));
    }

    #[test]
    fn test_is_ready() {
        assert!(!LifecycleState::Created.is_ready());
        assert!(!LifecycleState::Initialized.is_ready());
        assert!(LifecycleState::Active.is_ready());
        assert!(LifecycleState::Checking.is_ready());
        assert!(!LifecycleState::Destroyed.is_ready());
    }
}

