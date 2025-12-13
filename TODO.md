# Ferric Framework - TODO

A comprehensive list of Angular features to implement and work that is currently stubbed or incomplete.

## 🔴 Critical / Core Framework

### Component System
- [x] **Change Detection** - Zone-less change detection with:
  - [x] `ChangeDetectorRef` - Per-component change detection control
  - [x] `ApplicationRef` - Root-level change detection and app lifecycle
  - [x] `Scheduler` - Zone-less batched update scheduling
  - [x] OnPush strategy - Check only on input changes or explicit marking
  - [x] `markForCheck()` - Mark component and ancestors for check
  - [x] `detectChanges()` - Immediate synchronous change detection
  - [x] `detach()` / `reattach()` - Manual control over change detection
- [x] **Component Lifecycle Hooks** - Full implementation of:
  - [x] `on_init()` - After first change detection
  - [x] `on_changes()` - When input properties change
  - [x] `on_destroy()` - Cleanup before component removal
  - [x] `after_view_init()` - After view is initialized
  - [x] `after_content_init()` - After projected content is initialized
  - [x] `do_check()` - Custom change detection
  - [x] `LifecycleState` - State machine for component lifecycle
  - [x] `LifecycleManager` - Coordinated lifecycle management
  - [x] `CleanupRegistry` - Automatic cleanup on destroy
  - [x] Extended lifecycle traits (Async, Navigation, Visibility, Focus, Resize)
- [x] **Content Projection** (`<fe-content>`) - Project child content into component templates
  - [x] Default slot - `<fe-content></fe-content>`
  - [x] Named slots with selectors - `<fe-content select="[header]">`
  - [x] CSS selector matching (tag, class, attribute, combined)
  - [x] Fallback content - `<fe-content>Default</fe-content>`
  - [x] Multiple selectors - `select="[header], .title"`
  - [x] `ContentProjection` - Main projection handler
  - [x] `ProjectionContext` - Content assignment management
- [x] **View Encapsulation** - CSS scoping modes
  - [x] `Emulated` - Attribute-based scoping (default)
  - [x] `ShadowDom` - Native Shadow DOM isolation
  - [x] `None` - Global styles, no scoping
  - [x] `StyleEncapsulator` - CSS selector transformation
  - [x] `TemplateEncapsulator` - HTML attribute injection
  - [x] `ViewEncapsulator` - Combined encapsulator
  - [x] `:host` selector support
  - [x] `::fe-deep` / `::ng-deep` / `/deep/` support
  - [x] Style caching for performance
- [ ] **Dynamic Components** - `ComponentFactoryResolver` equivalent for runtime component creation

### Template System
- [x] **External File Templates** - `include_str!()` for HTML/CSS files at compile time
- [x] **Template Compiler** - Full implementation with:
  - [x] Property binding `[prop]="value"`
  - [x] Event binding `(event)="handler($event)"`
  - [x] Two-way binding `[(ngModel)]="value"`
  - [x] Interpolation `{{ expression }}`
  - [x] Attribute binding `[attr.name]="value"`
  - [x] Class binding `[class.active]="isActive"`
  - [x] Style binding `[style.color]="color"`
- [x] **Template References** - `#ref` syntax for referencing elements
- [x] **Template Context** - Context for expression evaluation with hierarchical scope
- [x] **Expression Evaluator** - Evaluate expressions with ternary, logical, comparison operators
- [x] **Template Instance** - Rendered templates with reactive bindings
- [x] **Structural Directives** - Implementations of:
  - [x] `*feIf` - Conditional rendering
  - [x] `*feFor` - List rendering with `$index`, `$first`, `$last`, `$even`, `$odd`
  - [x] `[feSwitch]` / `*feSwitchCase` / `*feSwitchDefault` - Switch case rendering
    - [x] `FeSwitchContainer` - Main switch container
    - [x] `SwitchCase` - Case with single or multiple values
    - [x] `SwitchValue` - Type-safe switch values (string, int, float, bool, null)
    - [x] Template parsing for switch directives
    - [x] Default case support
- [x] **Pipes** - Transform displayed values
  - [x] **Text Pipes**: `uppercase`, `lowercase`, `titlecase`, `trim`
  - [x] **Number Pipes**: `number`, `currency`, `percent`
  - [x] **Date Pipes**: `date` (short, medium, long, full, custom formats)
  - [x] **Utility Pipes**: `json`, `slice`, `default`, `replace`, `truncate`
  - [x] **String Pipes**: `padStart`, `padEnd`, `reverse`, `repeat`
  - [x] **Array Pipes**: `join`, `split`, `keyvalue`
  - [x] **Async Pipe**: `async` - Subscribe to promises/async values
    - [x] `AsyncState<T>` - Initial, Pending, Resolved, Error states
    - [x] `AsyncValue<T>` - Reactive async value with subscriptions
    - [x] `AsyncPipeRegistry` - Global cache for async values
    - [x] Helper functions: `async_pending()`, `async_resolved()`, `async_error()`
    - [x] Multiple JSON format support (`{state, value}`, `{loading, data}`, etc.)
  - [x] **Pipe Infrastructure**: `Pipe` trait, `PipeRegistry`, chaining
  - [x] Custom pipes via `impl Pipe`

### Dependency Injection
- [x] **Hierarchical Injectors** - Parent/child injector relationships with `Injector::child()`
- [x] **Injection Tokens** - `InjectionToken<T>` for non-class dependencies with `register_token()`
- [x] **Provider Scopes** - `providedIn: 'root'` / `'any'` with `ProvidedIn` enum and `SelfProviding` trait
- [x] **Optional Dependencies** - `Optional::new(&injector).resolve()` for optional resolution
- [x] **Self/SkipSelf** - `Self_::new()` and `SkipSelf::new()` for controlled resolution
- [x] **Multi Providers** - `MultiToken<T>` with `add_multi()` and `resolve_multi()`
- [x] **Factory Providers** - Custom factories with `register_factory()`
- [x] **Host Boundary** - `Host::new()` for component isolation
- [x] **Resolution Builder** - Fluent API with `Resolve::new(&injector).optional().skip_self().get()`

### Reactive System
- [x] **Effect Batching** - Batch multiple signal updates with `batch()`
- [x] **Computed Signals** - Lazy evaluation with automatic dependency tracking
- [x] **Signal Effects** - Side effects with automatic dependency tracking
- [x] **Resource API** - Async data fetching with loading/error states
- [x] **Dependency Tracking Runtime** - Automatic tracking of signal reads in effects/computed
- [x] **Watch Functions** - `watch()` and `watch_immediate()` for observing changes
- [x] **Derived Signals** - `derived()` for transforming signals
- [x] **ReadonlySignal** - Readonly views of signals
- [x] **Untracked Reads** - `untracked()` to read without creating dependencies

---

## 🟡 Important / Module Systems

### Router
- [x] **Route Activation** - Full `ActivatedRoute` implementation
  - [x] Reactive params via `Params` signal
  - [x] Reactive query params via `QueryParams` signal
  - [x] Fragment handling
  - [x] Route data and resolved data
  - [x] Parent/child route relationships
- [x] **Route Change Events** - Complete navigation event system
  - [x] `NavigationStart`, `NavigationEnd`, `NavigationCancel`, `NavigationError`
  - [x] `RoutesRecognized`, `GuardsCheckStart`, `GuardsCheckEnd`
  - [x] `ResolveStart`, `ResolveEnd`, `ActivationStart`, `ActivationEnd`
  - [x] `ChildActivationStart`, `ChildActivationEnd`, `RouteConfigLoadStart`, `RouteConfigLoadEnd`
  - [x] `Scroll` event for scroll position restoration
  - [x] `RouterEvents` with subscribe/emit pattern
- [x] **Lazy Loading** - Dynamic import of route modules
  - [x] `LazyRouteModule` - Module wrapper with load state tracking
  - [x] `load_children_path()` - Load children from JavaScript module path
  - [x] `load_children_factory()` - Load children from Rust factory function
  - [x] `LazyModuleRegistry` - Cache loaded modules for instant navigation
  - [x] `PreloadConfig` / `PreloadStrategy` - Configure preloading (All, None, OnDemand, Custom)
  - [x] `RouteConfigLoadStart` / `RouteConfigLoadEnd` events
  - [x] `navigate_async()` - Async navigation with lazy loading
  - [x] `preload()` / `preload_all()` - Manual preloading control
- [x] **Route Guards** - Full guard implementation
  - [x] `CanActivate` - Check before entering route
  - [x] `CanDeactivate<T>` - Check before leaving route
  - [x] `CanActivateChild` - Check before entering child routes
  - [x] `CanLoad` - Check before lazy loading
  - [x] Async guard support (`CanActivateAsync`, `CanDeactivateAsync`, `CanLoadAsync`)
  - [x] Built-in guards: `AlwaysAllow`, `AlwaysDeny`, `RedirectGuard`, `FunctionGuard`
  - [x] `CompositeGuard`, `AnyGuard` for combining guards
  - [x] `GuardRegistry` for named guard lookup
- [x] **Data Resolvers** - Pre-fetch data before route activation
  - [x] `Resolve` trait for sync resolvers
  - [x] `StaticResolver`, `ParamResolver`, `QueryParamResolver`
  - [x] `CompositeResolver` for multiple resolvers
  - [x] `ResolverRegistry` for named resolver lookup
- [x] **Child Routes** - Nested routing support
  - [x] Route tree matching
  - [x] Parent path prefix handling
- [x] **Named Outlets** - Multiple router outlets
  - [x] Outlet registration/unregistration
  - [x] Primary and named outlet support
- [x] **Route Params & Query Params** - Full parameter handling
  - [x] Path parameter extraction (`:paramName`)
  - [x] Wildcard params (`**path`)
  - [x] Query string parsing/serialization
  - [x] URL encoding/decoding
  - [x] Typed parameter access (`get_i32`, `get_bool`, etc.)
- [x] **Route Data** - Static route data
- [x] **URL Serialization** - Custom URL strategies
  - [x] `DefaultUrlSerializer` - Standard path-based routing
  - [x] `HashUrlSerializer` - Hash-based routing (`/#/path`)
  - [x] Custom serializers via `UrlSerializer` trait
- [x] **Router Links** - Declarative navigation
  - [x] `RouterLink` with query params and fragments
  - [x] `is_active()` and `is_exact_active()` checks

### Forms (`ferric_forms`)
- [x] **Form Group Typed Access** - Type-safe form group access
  - [x] `TypedFormGroup` trait with `get_value<T>()` method
  - [x] `TypedFormWrapper<T>` for compile-time type safety
  - [x] `FormValueResult<T>` for type-safe results
  - [x] `FormPath` for dot-notation access (e.g., `user.address.city`)
  - [x] `get_by_path()` for nested control access
- [x] **Form Array Operations** - Complete dynamic form arrays
  - [x] `push()`, `insert()`, `remove_at()`, `clear()`
  - [x] `move_control()` for reordering
  - [x] `at()`, `enumerate()`, `controls()`
- [x] **Cross-field Validation** - Validators that span multiple controls
  - [x] `CrossFieldValidatorTrait` for custom validators
  - [x] `MatchFieldsValidator` - Ensure two fields match
  - [x] `RequireOneOfValidator` - At least one field required
  - [x] `AllOrNoneValidator` - All or none have values
  - [x] `DateRangeValidator` - Start date before end date
  - [x] `NumericRangeValidator` - Min/max value validation
  - [x] `ConditionalRequiredValidator` - Required if condition met
  - [x] Helper functions: `password_match()`, `date_range()`, `require_one_of()`
- [x] **Async Validators** - Debounced server-side validation
  - [x] `AsyncValidator<T>` trait for async validation
  - [x] `AsyncValidatorBuilder` with fluent API
  - [x] `ConfiguredAsyncValidator` with debounce, cache, timeout
  - [x] `DebouncedValidator` wrapper
  - [x] `UniqueValidator` for uniqueness checks
  - [x] `ComposedAsyncValidator` for combining validators
  - [x] Helper functions: `unique_email()`, `unique_username()`
- [x] **Form Directives** - Template-driven form directives
  - [x] `FeModelDirective` - Two-way model binding `[(feModel)]`
  - [x] `FeFormGroupDirective` - Form group binding `[feFormGroup]`
  - [x] `FeFormControlNameDirective` - Control name binding `feFormControlName`
  - [x] `FeFormArrayNameDirective` - Array name binding `feFormArrayName`
  - [x] `FeFormControlDirective` - Direct control binding `[feFormControl]`
  - [x] `FormDirectiveContainer` for template integration
  - [x] `DirectiveAttributes` for parsing form directives
- [x] **ControlValueAccessor** - Custom form control integration
  - [x] `ControlValueAccessor` trait with `write_value`, `register_on_change`, etc.
  - [x] `ControlValueAccessorAny` for type-erased storage
  - [x] Built-in accessors: `DefaultValueAccessor`, `CheckboxValueAccessor`
  - [x] Built-in accessors: `NumberValueAccessor`, `SelectValueAccessor`
  - [x] Built-in accessors: `MultiSelectValueAccessor`, `RadioValueAccessor`
- [x] **Form Status** - `VALID`, `INVALID`, `PENDING`, `DISABLED` propagation
  - [x] `ControlStatus` enum with all states
  - [x] Status propagation from children to parent
  - [x] Cross-field validation affects group status
- [x] **`updateOn`** - Control when validation runs (`change`, `blur`, `submit`)
  - [x] `UpdateOn` enum: `Input`, `Change`, `Blur`, `Submit`
  - [x] Per-group configuration via `set_update_on()`
  - [x] `FormConfig` for form-level settings

### HTTP Client (`ferric-http`)
- [x] **HTTP Module** - Ergonomic HTTP client layer
  - [x] Platform abstraction (Fetch API for WASM, reqwest for native)
  - [x] Request builder pattern
  - [x] Response handling with typed bodies
  - [x] Headers management
  - [x] HTTP methods (GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS)
  - [x] Request/Response interceptors
    - [x] `Interceptor` with request/response handlers
    - [x] `InterceptorChain` for sequential processing
    - [x] `InterceptResult` (Continue, Abort, Skip)
    - [x] Built-in: `auth_interceptor`, `bearer_interceptor`, `logging_interceptor`
    - [x] Built-in: `header_interceptor`, `default_headers_interceptor`
    - [x] Built-in: `status_validator_interceptor`, `json_content_type_interceptor`
  - [x] Progress events
    - [x] `Progress` config with upload/download callbacks
    - [x] `ProgressInfo` with loaded, total, percent, fraction
    - [x] `ProgressTracker` with throttling support
    - [x] `ProgressEvent` enum for lifecycle tracking
  - [x] Request cancellation
    - [x] `CancellationToken` and `CancellationTrigger`
    - [x] `CancellationSource` for multiple tokens
    - [x] `CancelledFuture` for async cancellation
    - [x] `CancelOnDrop` guard for automatic cancellation
    - [x] `timeout_token()` for timeout-based cancellation
    - [x] `any_cancelled()`, `all_cancelled()` combinators
  - [x] Retry logic
    - [x] `RetryConfig` with max retries, delays, jitter
    - [x] `RetryStrategy` (Fixed, Linear, Exponential)
    - [x] `RetryPolicy` presets (none, fixed, exponential, aggressive, conservative)
    - [x] `RetryState` for tracking attempts
    - [x] `RetryExecutor` with callbacks
    - [x] Configurable retry status codes (408, 429, 500, 502, 503, 504)

---

## 🟢 Nice to Have / Advanced

### Testing
- [x] **TestBed** - Component testing utilities
  - [x] `TestBed::configure()` builder pattern
  - [x] `TestBedBuilder` with `component()`, `provide()`, `use_mock()`, `import()`
  - [x] `compile()` and `compile_async()` for test module creation
  - [x] `get<T>()`, `get_optional<T>()`, `get_required<T>()` for service resolution
  - [x] `create_component::<T>()` for fixture creation
  - [x] `TestSchema` for custom element validation
- [x] **Mock Services** - DI overrides for testing
  - [x] `Mock<T>` with expectation recording and verification
  - [x] `ExpectationBuilder` with `times()`, `once()`, `never()`, `with_args()`
  - [x] `Spy<T>` to wrap real services and record calls
  - [x] `Stub<T>` for predefined return values
  - [x] `MockResponseBuilder` for HTTP response mocking
  - [x] `MockResponseSequence` for retry testing
- [x] **Component Fixtures** - Render components in isolation
  - [x] `ComponentFixtureBuilder<T>` with `with_input()`, `with_provider()`
  - [x] `Fixture<T>` with input/output management
  - [x] `ElementQuery` for DOM querying with fluent assertions
  - [x] `ElementNode` and `DomBuilder` for test DOM construction
  - [x] `detect_changes()`, `when_stable()` for change detection
  - [x] Interaction methods: `click()`, `type_text()`, `set_value()`, `focus()`, `blur()`
- [x] **Async Testing** - Test async operations
  - [x] `FakeTimer` for time-based testing with `advance()`, `schedule()`
  - [x] `TestScheduler` for async task management
  - [x] `SignalTester<T>` for reactive value tracking
  - [x] `DeferredPromise<T>` with `resolve()`, `reject()`
  - [x] `StreamTester<T>` for stream testing
  - [x] `TestZone` for async stability tracking
  - [x] `wait_until()`, `yield_now()`, `with_timeout()` utilities
- [x] **Assertions** - Fluent test assertions
  - [x] `assert_that()` with chainable methods
  - [x] `SoftAssertions` for multiple failure collection
  - [x] Type-specific assertions (bool, Option, Result, String, Vec)
- [x] **DOM Testing** - DOM simulation and assertions
  - [x] `TestDom` with query selectors
  - [x] `DomNode` with attributes, classes, children management
  - [x] `TestEvent` for event simulation (click, input, keydown)

### Benchmarks
- [x] **React vs Ferric Todo Comparison** - `benchmarks/todo-comparison/`
  - [x] Bundle size measurement (raw, gzip)
  - [x] Runtime performance (add, toggle, filter, clear)
  - [x] Memory usage tracking
  - [x] Puppeteer-based automated benchmarks
  - [x] Markdown report generation

### Animations
- [ ] **Animation Module** - CSS/JS animations
  - Triggers and states
  - Transitions
  - Keyframes
  - Stagger animations
  - Route animations

### i18n (Internationalization) - `ferric-core::i18n`
- [x] **I18nService** - Injectable service for DI integration
- [x] **Translation Service** - Runtime translations with JSON loading
- [x] **Parameter Interpolation** - `"Hello, {name}!"` syntax
- [x] **Locale Pipes** - Number, currency formatting
- [x] **Date Formatting** - Short, Medium, Long, Full, ISO formats
- [x] **Pluralization** - CLDR-based plural rules for 30+ languages
- [x] **Locale Detection** - Browser locale detection
- [x] **Fallback Chain** - Automatic locale fallback (e.g., zh-Hans-CN → zh-Hans → zh → en)
- [x] **RTL Detection** - Right-to-left language support
- [x] **ICU Message Format** - Full ICU support
  - [x] `IcuMessage::parse()` - Parse ICU message strings
  - [x] Plural forms: `{count, plural, one{# item} other{# items}}`
  - [x] Select forms: `{gender, select, male{He} female{She} other{They}}`
  - [x] Selectordinal: `{n, selectordinal, one{#st} two{#nd} few{#rd} other{#th}}`
  - [x] Number formatting: `{amount, number}`, `{amount, number, currency}`
  - [x] Date/time formatting: `{date, date, short}`, `{time, time}`
  - [x] Nested messages: Plural inside select and vice versa
  - [x] Escaping: `''` for literal quote, `'{...'` for literal braces
  - [x] Offset in plural: `{count, plural, offset:1 =0{no items} one{# item}}`
  - [x] Exact value matching: `=0{zero}`, `=1{one}`
  - [x] `IcuValue` - Type-safe argument values (String, Number, Date, Bool)
  - [x] `I18nService.icu()` / `I18nService.t_icu()` - ICU formatting methods
  - [x] `icu_compile()` / `icu_format()` - Standalone helper functions

### DevTools
- [x] **Debug Mode** - Development helpers
  - [x] `DevMode::enable()` / `DevMode::disable()` - Toggle debug mode
  - [x] `DebugConfig` - Configurable logging (lifecycle, signals, routing, DI)
  - [x] `debug_log!`, `debug_warn!`, `debug_error!` - Conditional logging macros
  - [x] `DebugValidator` - Validation helpers for development
  - [x] `DebugCounter`, `DebugTimer` - Debugging utilities
  - [x] `breakpoint()` - Debug breakpoint support
- [x] **Component Inspector** - Browser devtools integration
  - [x] `ComponentNode` - Component tree representation
  - [x] `ComponentInspector` - Tree inspection and selection
  - [x] `highlight()` / `clear_highlight()` - DOM highlighting
  - [x] Custom events for devtools communication
  - [x] `__FERRIC_DEVTOOLS__` global for browser access
  - [x] `InjectorNode`, `ProviderInfo` - DI inspection
- [x] **Performance Profiler** - Change detection profiling
  - [x] `Profiler` - Main profiling interface
  - [x] `ProfilerSettings` - Configurable profiling options
  - [x] `record_change_detection()`, `record_render()`, `record_effect()`, `record_signal_update()`
  - [x] `FrameProfile` - Per-frame performance data
  - [x] `ComponentStats` - Per-component statistics
  - [x] `ProfileReport` with summary, slowest components, most rendered
  - [x] `ProfileGuard` - RAII-style profiling
- [x] **Console Utilities** - Structured browser console output
  - [x] `Console` - Cross-platform console methods
  - [x] `Logger` - Leveled logging with history
  - [x] `StyledLog` - Rich styled console output
  - [x] `ConsoleGroup`, `ConsoleTimer` - RAII guards
- [x] **State Inspection** - Runtime state debugging
  - [x] `StateInspector` - Track state changes
  - [x] `StateSnapshot` - Point-in-time state capture
  - [x] `StateDiff` - Compare snapshots
  - [x] `SignalTracker` - Signal dependency visualization
  - [x] `TimeTravelDebugger` - Navigate state history

### Server-Side Rendering (`ferric-ssr`)
- [x] **SSR Support** - Pre-render components
  - [x] `SsrServer` - Hyper-based HTTP server
  - [x] `Renderable` trait - Component rendering interface
  - [x] `RenderContext` - Rendering configuration
  - [x] `HtmlRenderer` - HTML output generation
  - [x] `SerializedState` - State serialization for hydration
  - [x] `HtmlStream` - Progressive/streaming rendering
  - [x] Platform detection (server vs browser)
- [ ] **Full Hydration** - Rehydrate server-rendered content with full interactivity
- [ ] **Partial Hydration** - Island architecture support

---

## 📝 Code Cleanup / Technical Debt

### Unused Code (from compiler warnings)
- [ ] Clean up unused functions in `ferric-core/src/utils.rs`:
  - `log()`, `error()`, `warn()`
- [ ] Clean up unused macros utilities in `ferric_macros/src/utils.rs`:
  - `to_pascal_case()`, `to_snake_case()`, `to_camel_case()`
  - `ident()`, `extract_option_inner()`, `extract_vec_inner()`
  - `unique_ident()`, `parse_binding()`, `BindingKind`
- [ ] Implement or remove `ViewChildArgs` struct
- [ ] Use or remove `style_urls` in `ComponentArgs`
- [ ] Implement subscription tracking in `EffectInner`
- [ ] Implement `base_href` in `RouterInner`

### Structural Directives (Stubbed)
- [ ] `ForDirective` - Fields `template`, `view_container`, `views` are never used
- [ ] Complete directive view management

### Forms Technical Debt
- [ ] `DebouncedValidator.inner` is never read
- [ ] `ValidateField.ty` and `.custom` are never read in derive macro

### IndexedDB (`ferric_idb`)
- [ ] `KeyRangeBuilder` is never constructed - consider removing or documenting
- [ ] `IndexBuilder::new()` is never used

---

## 🎯 Immediate Next Steps (Prioritized)

1. ~~**Template Compiler** - The backbone of Angular-style templates~~ ✅ Done
2. ~~**Change Detection** - Without this, reactivity doesn't flow to templates~~ ✅ Done
3. ~~**Component Lifecycle** - Essential for real-world components~~ ✅ Done
4. ~~**i18n** - Internationalization support~~ ✅ Done
5. ~~**Content Projection** - `<fe-content>` for component composition~~ ✅ Done
6. ~~**View Encapsulation** - CSS scoping modes~~ ✅ Done
7. **Route Activation** - Complete the router for SPA functionality
8. **Pipes** - Transform displayed values in templates
9. **Dynamic Components** - Runtime component creation

---

## 🛠️ CLI (`ferric-cli`)

- [x] **Project Scaffolding** - `ferric new <name>` creates a new Ferric project
- [x] **Component Generation** - `ferric generate component <name>` with:
  - [x] External HTML templates (default)
  - [x] External SCSS styles (default)
  - [x] BEM naming convention
  - [x] Test file generation
- [x] **Service Generation** - `ferric generate service <name>`
- [x] **Directive Generation** - `ferric generate directive <name>`
- [x] **Pipe Generation** - `ferric generate pipe <name>`
- [x] **Module Generation** - `ferric generate module <name>`
- [x] **Guard Generation** - `ferric generate guard <name>`
- [x] **Build Command** - `ferric build` with:
  - [x] WASM compilation via wasm-pack
  - [x] SCSS to CSS compilation via grass
  - [x] TailwindCSS integration via PostCSS
  - [x] Asset bundling
  - [x] Watch mode
  - [x] Release optimization
- [x] **Dev Server** - `ferric serve` with:
  - [x] Live reload
  - [x] File watching
  - [x] Automatic rebuild
  - [x] TailwindCSS watch mode
- [x] **TailwindCSS Commands** - `ferric tailwind` / `ferric tailwind-init`

---

## 📚 Angular Feature Parity Reference

| Feature | Angular | Ferric Status |
|---------|---------|---------------|
| Components | ✅ | 🟢 Implemented |
| Templates | ✅ | 🟢 Implemented |
| Directives | ✅ | 🟡 Partial |
| Services/DI | ✅ | 🟢 Implemented |
| Routing | ✅ | 🟡 Partial |
| Forms | ✅ | 🟢 Implemented |
| HTTP | ✅ | 🟡 Partial |
| Pipes | ✅ | 🟢 Implemented |
| Animations | ✅ | 🔴 Missing |
| i18n | ✅ | 🟢 Implemented |
| Testing | ✅ | 🟢 Implemented |
| SSR | ✅ | 🟡 Partial |
| Signals | ✅ | 🟢 Implemented |
| Standalone | ✅ | 🟢 Default |
| CLI | ✅ | 🟢 Implemented |

**Legend:**
- 🟢 Implemented
- 🟡 Partial / In Progress
- 🔴 Missing / Stubbed
