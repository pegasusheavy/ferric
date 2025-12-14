//! Framework-wide configuration tokens and module.

use crate::di::{InjectionToken, Injector, Injectable};

/// Configuration tokens for core framework features.
pub mod tokens {
    use super::*;

    /// Enable change detection debugging.
    pub const DEBUG_CHANGE_DETECTION: InjectionToken<bool> =
        InjectionToken::with_id("DEBUG_CHANGE_DETECTION", 5001);

    /// Enable lifecycle hook debugging.
    pub const DEBUG_LIFECYCLE: InjectionToken<bool> =
        InjectionToken::with_id("DEBUG_LIFECYCLE", 5002);

    /// Enable DI resolution debugging.
    pub const DEBUG_DI: InjectionToken<bool> =
        InjectionToken::with_id("DEBUG_DI", 5003);

    /// Production mode (disables debugging, enables optimizations).
    pub const PRODUCTION_MODE: InjectionToken<bool> =
        InjectionToken::with_id("PRODUCTION_MODE", 5004);

    /// Application name.
    pub const APP_NAME: InjectionToken<String> =
        InjectionToken::with_id("APP_NAME", 5005);

    /// Application version.
    pub const APP_VERSION: InjectionToken<String> =
        InjectionToken::with_id("APP_VERSION", 5006);

    /// Environment name (e.g., "development", "production").
    pub const ENVIRONMENT: InjectionToken<String> =
        InjectionToken::with_id("ENVIRONMENT", 5007);

    /// Locale for internationalization.
    pub const LOCALE: InjectionToken<String> =
        InjectionToken::with_id("LOCALE", 5008);

    /// Timezone for date/time formatting.
    pub const TIMEZONE: InjectionToken<String> =
        InjectionToken::with_id("TIMEZONE", 5009);
}

/// Framework configuration.
pub struct FrameworkConfig {
    production_mode: bool,
    debug_change_detection: bool,
    debug_lifecycle: bool,
    debug_di: bool,
    app_name: String,
    environment: String,
}

impl Default for FrameworkConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl FrameworkConfig {
    pub fn new() -> Self {
        Self {
            production_mode: false,
            debug_change_detection: false,
            debug_lifecycle: false,
            debug_di: false,
            app_name: "FerricApp".to_string(),
            environment: "development".to_string(),
        }
    }

    pub fn production() -> Self {
        Self {
            production_mode: true,
            debug_change_detection: false,
            debug_lifecycle: false,
            debug_di: false,
            app_name: "FerricApp".to_string(),
            environment: "production".to_string(),
        }
    }

    pub fn is_production(&self) -> bool {
        self.production_mode
    }

    pub fn is_debug_change_detection(&self) -> bool {
        self.debug_change_detection
    }

    pub fn is_debug_lifecycle(&self) -> bool {
        self.debug_lifecycle
    }

    pub fn is_debug_di(&self) -> bool {
        self.debug_di
    }

    pub fn app_name(&self) -> &str {
        &self.app_name
    }

    pub fn environment(&self) -> &str {
        &self.environment
    }
}

impl Injectable for FrameworkConfig {
    fn create(injector: &Injector) -> Self {
        let mut config = Self::new();

        if let Some(prod) = injector.resolve_token(&tokens::PRODUCTION_MODE) {
            config.production_mode = *prod;
        }

        if let Some(debug_cd) = injector.resolve_token(&tokens::DEBUG_CHANGE_DETECTION) {
            config.debug_change_detection = *debug_cd;
        }

        if let Some(debug_lifecycle) = injector.resolve_token(&tokens::DEBUG_LIFECYCLE) {
            config.debug_lifecycle = *debug_lifecycle;
        }

        if let Some(debug_di) = injector.resolve_token(&tokens::DEBUG_DI) {
            config.debug_di = *debug_di;
        }

        if let Some(name) = injector.resolve_token(&tokens::APP_NAME) {
            config.app_name = (*name).clone();
        }

        if let Some(env) = injector.resolve_token(&tokens::ENVIRONMENT) {
            config.environment = (*env).clone();
        }

        config
    }
}

/// Core framework module.
pub struct CoreModule;

impl CoreModule {
    /// Provide framework with configuration.
    pub fn provide(injector: &Injector) {
        injector.register_singleton::<FrameworkConfig>();
    }

    /// Provide for production environment.
    pub fn provide_for_production(injector: &Injector) {
        injector.register_token(&tokens::PRODUCTION_MODE, true);
        injector.register_token(&tokens::ENVIRONMENT, "production".to_string());
        injector.register_singleton::<FrameworkConfig>();
    }

    /// Provide for development environment.
    pub fn provide_for_development(injector: &Injector) {
        injector.register_token(&tokens::PRODUCTION_MODE, false);
        injector.register_token(&tokens::DEBUG_CHANGE_DETECTION, true);
        injector.register_token(&tokens::DEBUG_LIFECYCLE, true);
        injector.register_token(&tokens::ENVIRONMENT, "development".to_string());
        injector.register_singleton::<FrameworkConfig>();
    }
}

