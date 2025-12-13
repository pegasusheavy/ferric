//! Navigation events for the router.
//!
//! Provides a comprehensive event system for tracking navigation lifecycle.
//!
//! ## Event Sequence
//!
//! 1. `NavigationStart` - Navigation begins
//! 2. `RouteConfigLoadStart` - Route config loading begins (lazy routes)
//! 3. `RouteConfigLoadEnd` - Route config loaded
//! 4. `RoutesRecognized` - Routes matched
//! 5. `GuardsCheckStart` - Guard checks begin
//! 6. `ChildActivationStart` - Child route activation begins
//! 7. `ChildActivationEnd` - Child route activation ends
//! 8. `GuardsCheckEnd` - Guard checks complete
//! 9. `ResolveStart` - Data resolvers begin
//! 10. `ResolveEnd` - Data resolvers complete
//! 11. `ActivationStart` - Route activation begins
//! 12. `ActivationEnd` - Route activation ends
//! 13. `NavigationEnd` - Navigation complete (or `NavigationCancel`/`NavigationError`)

use super::{ActivatedRoute, Route};
use crate::reactive::{signal, Signal};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Unique ID for navigation events.
pub type NavigationId = u64;

/// Base trait for all router events.
pub trait RouterEvent: std::fmt::Debug {
    /// Get the navigation ID.
    fn id(&self) -> NavigationId;

    /// Get the target URL.
    fn url(&self) -> &str;
}

/// Navigation has started.
#[derive(Debug, Clone)]
pub struct NavigationStart {
    /// Unique navigation ID.
    pub id: NavigationId,
    /// Target URL.
    pub url: String,
    /// Navigation trigger (imperative, popstate, hashchange, etc.).
    pub trigger: NavigationTrigger,
    /// State passed during navigation.
    pub state: Option<NavigationState>,
}

impl RouterEvent for NavigationStart {
    fn id(&self) -> NavigationId {
        self.id
    }
    fn url(&self) -> &str {
        &self.url
    }
}

/// Navigation has completed successfully.
#[derive(Debug, Clone)]
pub struct NavigationEnd {
    /// Navigation ID.
    pub id: NavigationId,
    /// Final URL after navigation.
    pub url: String,
    /// Final URL after redirects.
    pub url_after_redirects: String,
}

impl RouterEvent for NavigationEnd {
    fn id(&self) -> NavigationId {
        self.id
    }
    fn url(&self) -> &str {
        &self.url
    }
}

/// Navigation was cancelled.
#[derive(Debug, Clone)]
pub struct NavigationCancel {
    /// Navigation ID.
    pub id: NavigationId,
    /// URL that was being navigated to.
    pub url: String,
    /// Reason for cancellation.
    pub reason: CancelReason,
}

impl RouterEvent for NavigationCancel {
    fn id(&self) -> NavigationId {
        self.id
    }
    fn url(&self) -> &str {
        &self.url
    }
}

/// Navigation failed with an error.
#[derive(Debug, Clone)]
pub struct NavigationError {
    /// Navigation ID.
    pub id: NavigationId,
    /// URL that was being navigated to.
    pub url: String,
    /// Error message.
    pub error: String,
}

impl RouterEvent for NavigationError {
    fn id(&self) -> NavigationId {
        self.id
    }
    fn url(&self) -> &str {
        &self.url
    }
}

/// Routes have been recognized and matched.
#[derive(Debug, Clone)]
pub struct RoutesRecognized {
    /// Navigation ID.
    pub id: NavigationId,
    /// URL being navigated to.
    pub url: String,
    /// Matched route state.
    pub state: RouterState,
}

impl RouterEvent for RoutesRecognized {
    fn id(&self) -> NavigationId {
        self.id
    }
    fn url(&self) -> &str {
        &self.url
    }
}

/// Guard check has started.
#[derive(Debug, Clone)]
pub struct GuardsCheckStart {
    /// Navigation ID.
    pub id: NavigationId,
    /// URL being navigated to.
    pub url: String,
    /// Router state being checked.
    pub state: RouterState,
}

impl RouterEvent for GuardsCheckStart {
    fn id(&self) -> NavigationId {
        self.id
    }
    fn url(&self) -> &str {
        &self.url
    }
}

/// Guard check has completed.
#[derive(Debug, Clone)]
pub struct GuardsCheckEnd {
    /// Navigation ID.
    pub id: NavigationId,
    /// URL being navigated to.
    pub url: String,
    /// Whether guards allowed navigation.
    pub should_activate: bool,
}

impl RouterEvent for GuardsCheckEnd {
    fn id(&self) -> NavigationId {
        self.id
    }
    fn url(&self) -> &str {
        &self.url
    }
}

/// Data resolvers have started.
#[derive(Debug, Clone)]
pub struct ResolveStart {
    /// Navigation ID.
    pub id: NavigationId,
    /// URL being navigated to.
    pub url: String,
    /// Router state being resolved.
    pub state: RouterState,
}

impl RouterEvent for ResolveStart {
    fn id(&self) -> NavigationId {
        self.id
    }
    fn url(&self) -> &str {
        &self.url
    }
}

/// Data resolvers have completed.
#[derive(Debug, Clone)]
pub struct ResolveEnd {
    /// Navigation ID.
    pub id: NavigationId,
    /// URL being navigated to.
    pub url: String,
    /// Router state after resolution.
    pub state: RouterState,
}

impl RouterEvent for ResolveEnd {
    fn id(&self) -> NavigationId {
        self.id
    }
    fn url(&self) -> &str {
        &self.url
    }
}

/// Route activation has started.
#[derive(Debug, Clone)]
pub struct ActivationStart {
    /// Navigation ID.
    pub id: NavigationId,
    /// URL being navigated to.
    pub url: String,
    /// Snapshot of the route being activated.
    pub snapshot: ActivatedRouteSnapshot,
}

impl RouterEvent for ActivationStart {
    fn id(&self) -> NavigationId {
        self.id
    }
    fn url(&self) -> &str {
        &self.url
    }
}

/// Route activation has completed.
#[derive(Debug, Clone)]
pub struct ActivationEnd {
    /// Navigation ID.
    pub id: NavigationId,
    /// URL navigated to.
    pub url: String,
    /// Snapshot of the activated route.
    pub snapshot: ActivatedRouteSnapshot,
}

impl RouterEvent for ActivationEnd {
    fn id(&self) -> NavigationId {
        self.id
    }
    fn url(&self) -> &str {
        &self.url
    }
}

/// Child route activation has started.
#[derive(Debug, Clone)]
pub struct ChildActivationStart {
    /// Navigation ID.
    pub id: NavigationId,
    /// URL being navigated to.
    pub url: String,
    /// Snapshot of the route being activated.
    pub snapshot: ActivatedRouteSnapshot,
}

impl RouterEvent for ChildActivationStart {
    fn id(&self) -> NavigationId {
        self.id
    }
    fn url(&self) -> &str {
        &self.url
    }
}

/// Child route activation has completed.
#[derive(Debug, Clone)]
pub struct ChildActivationEnd {
    /// Navigation ID.
    pub id: NavigationId,
    /// URL navigated to.
    pub url: String,
    /// Snapshot of the activated route.
    pub snapshot: ActivatedRouteSnapshot,
}

impl RouterEvent for ChildActivationEnd {
    fn id(&self) -> NavigationId {
        self.id
    }
    fn url(&self) -> &str {
        &self.url
    }
}

/// Lazy route configuration loading has started.
#[derive(Debug, Clone)]
pub struct RouteConfigLoadStart {
    /// Navigation ID.
    pub id: NavigationId,
    /// URL being navigated to.
    pub url: String,
    /// Route being loaded.
    pub route: Route,
}

impl RouterEvent for RouteConfigLoadStart {
    fn id(&self) -> NavigationId {
        self.id
    }
    fn url(&self) -> &str {
        &self.url
    }
}

/// Lazy route configuration loading has completed.
#[derive(Debug, Clone)]
pub struct RouteConfigLoadEnd {
    /// Navigation ID.
    pub id: NavigationId,
    /// URL being navigated to.
    pub url: String,
    /// Route that was loaded.
    pub route: Route,
}

impl RouterEvent for RouteConfigLoadEnd {
    fn id(&self) -> NavigationId {
        self.id
    }
    fn url(&self) -> &str {
        &self.url
    }
}

/// Scroll position restored or scrolled to anchor.
#[derive(Debug, Clone)]
pub struct Scroll {
    /// Navigation ID.
    pub id: NavigationId,
    /// URL navigated to.
    pub url: String,
    /// Scroll position (x, y).
    pub position: Option<(i32, i32)>,
    /// Anchor to scroll to.
    pub anchor: Option<String>,
}

impl RouterEvent for Scroll {
    fn id(&self) -> NavigationId {
        self.id
    }
    fn url(&self) -> &str {
        &self.url
    }
}

/// All router events as an enum.
#[derive(Debug, Clone)]
pub enum Event {
    NavigationStart(NavigationStart),
    NavigationEnd(NavigationEnd),
    NavigationCancel(NavigationCancel),
    NavigationError(NavigationError),
    RoutesRecognized(RoutesRecognized),
    GuardsCheckStart(GuardsCheckStart),
    GuardsCheckEnd(GuardsCheckEnd),
    ResolveStart(ResolveStart),
    ResolveEnd(ResolveEnd),
    ActivationStart(ActivationStart),
    ActivationEnd(ActivationEnd),
    ChildActivationStart(ChildActivationStart),
    ChildActivationEnd(ChildActivationEnd),
    RouteConfigLoadStart(RouteConfigLoadStart),
    RouteConfigLoadEnd(RouteConfigLoadEnd),
    Scroll(Scroll),
}

impl Event {
    /// Get the navigation ID from any event.
    pub fn id(&self) -> NavigationId {
        match self {
            Event::NavigationStart(e) => e.id,
            Event::NavigationEnd(e) => e.id,
            Event::NavigationCancel(e) => e.id,
            Event::NavigationError(e) => e.id,
            Event::RoutesRecognized(e) => e.id,
            Event::GuardsCheckStart(e) => e.id,
            Event::GuardsCheckEnd(e) => e.id,
            Event::ResolveStart(e) => e.id,
            Event::ResolveEnd(e) => e.id,
            Event::ActivationStart(e) => e.id,
            Event::ActivationEnd(e) => e.id,
            Event::ChildActivationStart(e) => e.id,
            Event::ChildActivationEnd(e) => e.id,
            Event::RouteConfigLoadStart(e) => e.id,
            Event::RouteConfigLoadEnd(e) => e.id,
            Event::Scroll(e) => e.id,
        }
    }

    /// Get the URL from any event.
    pub fn url(&self) -> &str {
        match self {
            Event::NavigationStart(e) => &e.url,
            Event::NavigationEnd(e) => &e.url,
            Event::NavigationCancel(e) => &e.url,
            Event::NavigationError(e) => &e.url,
            Event::RoutesRecognized(e) => &e.url,
            Event::GuardsCheckStart(e) => &e.url,
            Event::GuardsCheckEnd(e) => &e.url,
            Event::ResolveStart(e) => &e.url,
            Event::ResolveEnd(e) => &e.url,
            Event::ActivationStart(e) => &e.url,
            Event::ActivationEnd(e) => &e.url,
            Event::ChildActivationStart(e) => &e.url,
            Event::ChildActivationEnd(e) => &e.url,
            Event::RouteConfigLoadStart(e) => &e.url,
            Event::RouteConfigLoadEnd(e) => &e.url,
            Event::Scroll(e) => &e.url,
        }
    }

    /// Check if this is a NavigationEnd event.
    pub fn is_navigation_end(&self) -> bool {
        matches!(self, Event::NavigationEnd(_))
    }

    /// Check if this is a NavigationStart event.
    pub fn is_navigation_start(&self) -> bool {
        matches!(self, Event::NavigationStart(_))
    }
}

/// What triggered the navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationTrigger {
    /// Programmatic navigation via router.navigate().
    Imperative,
    /// Browser back/forward button.
    PopState,
    /// Hash change in URL.
    HashChange,
    /// Initial page load.
    Initial,
}

/// Reason for navigation cancellation.
#[derive(Debug, Clone)]
pub enum CancelReason {
    /// Guard rejected navigation.
    GuardRejected,
    /// Navigation was superseded by another navigation.
    Superseded,
    /// User cancelled navigation.
    User,
    /// Redirect occurred.
    Redirect(String),
}

/// State passed during navigation.
#[derive(Debug, Clone, Default)]
pub struct NavigationState {
    /// Arbitrary data passed with navigation.
    pub data: HashMap<String, String>,
}

/// Snapshot of router state at a point in time.
#[derive(Debug, Clone, Default)]
pub struct RouterState {
    /// Root activated route snapshot.
    pub root: Option<ActivatedRouteSnapshot>,
    /// URL of this state.
    pub url: String,
}

/// Snapshot of an activated route.
#[derive(Debug, Clone)]
pub struct ActivatedRouteSnapshot {
    /// Route path pattern.
    pub path: String,
    /// URL parameters.
    pub params: HashMap<String, String>,
    /// Query parameters.
    pub query_params: HashMap<String, String>,
    /// URL fragment.
    pub fragment: Option<String>,
    /// Static route data.
    pub data: HashMap<String, String>,
    /// Resolved data.
    pub resolved_data: HashMap<String, String>,
    /// Component selector.
    pub component: Option<String>,
    /// Child route snapshots.
    pub children: Vec<ActivatedRouteSnapshot>,
    /// Outlet name this route renders to.
    pub outlet: String,
}

impl Default for ActivatedRouteSnapshot {
    fn default() -> Self {
        Self {
            path: String::new(),
            params: HashMap::new(),
            query_params: HashMap::new(),
            fragment: None,
            data: HashMap::new(),
            resolved_data: HashMap::new(),
            component: None,
            children: Vec::new(),
            outlet: "primary".to_string(),
        }
    }
}

/// Event emitter for router events.
pub struct RouterEvents {
    /// Subscribers to events.
    subscribers: RefCell<Vec<Box<dyn Fn(&Event)>>>,
    /// Last emitted event for replay.
    last_event: RefCell<Option<Event>>,
    /// Event history (optional, limited size).
    history: RefCell<Vec<Event>>,
    /// Maximum history size.
    max_history: usize,
}

impl RouterEvents {
    /// Create a new event emitter.
    pub fn new() -> Self {
        Self {
            subscribers: RefCell::new(Vec::new()),
            last_event: RefCell::new(None),
            history: RefCell::new(Vec::new()),
            max_history: 50,
        }
    }

    /// Subscribe to all events.
    pub fn subscribe<F: Fn(&Event) + 'static>(&self, callback: F) -> SubscriptionId {
        let id = self.subscribers.borrow().len();
        self.subscribers.borrow_mut().push(Box::new(callback));
        SubscriptionId(id)
    }

    /// Emit an event to all subscribers.
    pub fn emit(&self, event: Event) {
        // Store in history
        {
            let mut history = self.history.borrow_mut();
            history.push(event.clone());
            if history.len() > self.max_history {
                history.remove(0);
            }
        }

        // Store last event
        *self.last_event.borrow_mut() = Some(event.clone());

        // Notify subscribers
        for subscriber in self.subscribers.borrow().iter() {
            subscriber(&event);
        }
    }

    /// Get the last emitted event.
    pub fn last_event(&self) -> Option<Event> {
        self.last_event.borrow().clone()
    }

    /// Get event history.
    pub fn history(&self) -> Vec<Event> {
        self.history.borrow().clone()
    }

    /// Clear history.
    pub fn clear_history(&self) {
        self.history.borrow_mut().clear();
    }
}

impl Default for RouterEvents {
    fn default() -> Self {
        Self::new()
    }
}

/// Subscription ID for unsubscribing.
#[derive(Debug, Clone, Copy)]
pub struct SubscriptionId(usize);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navigation_start_event() {
        let event = NavigationStart {
            id: 1,
            url: "/users".to_string(),
            trigger: NavigationTrigger::Imperative,
            state: None,
        };

        assert_eq!(event.id(), 1);
        assert_eq!(event.url(), "/users");
    }

    #[test]
    fn test_event_enum() {
        let event = Event::NavigationStart(NavigationStart {
            id: 42,
            url: "/home".to_string(),
            trigger: NavigationTrigger::Initial,
            state: None,
        });

        assert_eq!(event.id(), 42);
        assert!(event.is_navigation_start());
    }

    #[test]
    fn test_router_events_emit() {
        let events = RouterEvents::new();
        let received = Rc::new(RefCell::new(Vec::new()));

        let received_clone = received.clone();
        events.subscribe(move |e| {
            received_clone.borrow_mut().push(e.id());
        });

        events.emit(Event::NavigationStart(NavigationStart {
            id: 1,
            url: "/test".to_string(),
            trigger: NavigationTrigger::Imperative,
            state: None,
        }));

        assert_eq!(received.borrow().len(), 1);
        assert_eq!(received.borrow()[0], 1);
    }
}
