//! Showcase of all new Ferric macros.
//!
//! This example demonstrates the usage of all newly added utility macros
//! in both ferric-macros and ferric-forms-macros crates.

use ferric_core::prelude::*;
use ferric_forms::prelude::*;

// ============================================================================
// REACTIVE MACROS SHOWCASE
// ============================================================================

/// Demonstrates reactive programming macros.
pub fn reactive_showcase() {
    // signal! - Create reactive state
    let count = signal!(0);
    let name = signal!("Alice".to_string());

    // computed! - Derived values
    let doubled = computed!(|| count.get() * 2);
    let greeting = computed!(|| format!("Hello, {}!", name.get()));

    // effect! - Side effects
    effect!(|| {
        web_sys::console::log_1(&format!("Count is: {}", count.get()).into());
    });

    // batch! - Batch updates
    batch!(|| {
        count.set(10);
        name.set("Bob".to_string());
    });

    // memo! - Memoized computation (cached)
    let expensive_result = memo!(|| {
        // Expensive computation
        (1..=100).sum::<i32>()
    });

    // watch! - Watch for changes
    watch!(|| {
        if count.get() > 50 {
            web_sys::console::log_1(&"Count exceeded 50!".into());
        }
    });

    web_sys::console::log_1(&format!("Doubled: {}", doubled.get()).into());
    web_sys::console::log_1(&format!("Greeting: {}", greeting.get()).into());
    web_sys::console::log_1(&format!("Memoized result: {}", expensive_result).into());
}

// ============================================================================
// TEMPLATE MACROS SHOWCASE
// ============================================================================

/// Demonstrates template-related macros.
pub fn template_showcase() {
    // html! - Parse HTML
    let template = html!(r#"
        <div class="container">
            <h1>{{ title }}</h1>
            <p>{{ content }}</p>
        </div>
    "#);

    // css! - Define CSS
    let styles = css!(r#"
        .container {
            padding: 20px;
            background: #f5f5f5;
        }
        h1 {
            color: #333;
            margin-bottom: 10px;
        }
    "#);

    // selector! - Validate selector
    let selector = selector!("app-showcase");

    web_sys::console::log_1(&format!("Selector: {}", selector).into());
}

// ============================================================================
// FORMS MACROS SHOWCASE
// ============================================================================

/// User model for type-safe forms.
#[derive(TypedForm, Clone)]
pub struct User {
    pub username: String,
    pub email: String,
    pub age: u32,
}

/// Demonstrates form-related macros.
pub fn forms_showcase() {
    // build_form! - Declarative form building
    let login_form = build_form! {
        email: FormControl::text("").with_validators(compose_validators![
            required(),
            email(),
        ]),
        password: FormControl::password("").with_validators(compose_validators![
            required(),
            min_length(8),
        ]),
    };

    // TypedForm - Type-safe forms
    let user = User {
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
        age: 25,
    };

    let user_form = UserForm::from_value(user.clone());

    // Modify form values
    user_form.username.set_value("bob".to_string());
    user_form.age.set_value(30);

    // Extract back to User
    let updated_user: User = user_form.into();

    web_sys::console::log_1(&format!("Updated username: {}", updated_user.username).into());
    web_sys::console::log_1(&format!("Login form valid: {}", login_form.is_valid()).into());
}

/// Demonstrates reactive forms.
pub fn reactive_forms_showcase() {
    // reactive_form! - Forms with signal integration
    let form = reactive_form! {
        name: FormControl::text(""),
        email: FormControl::text(""),
        age: FormControl::number(0),
    };

    // React to form changes
    effect!(|| {
        let form_value = form.get();
        if form_value.is_valid() {
            web_sys::console::log_1(&"Form is valid!".into());
        } else {
            web_sys::console::log_1(&"Form has errors".into());
        }
    });

    // Update form values
    batch!(|| {
        form.get().get_control("name").unwrap().set_value("Alice");
        form.get().get_control("email").unwrap().set_value("alice@example.com");
        form.get().get_control("age").unwrap().set_value(25);
    });
}

// ============================================================================
// VALIDATOR MACROS SHOWCASE
// ============================================================================

/// Registration form with conditional validation.
pub struct RegistrationForm {
    form: FormGroup,
    is_company: Signal<bool>,
}

impl RegistrationForm {
    pub fn new() -> Self {
        let is_company = signal!(false);

        let form = build_form! {
            username: FormControl::text("").with_validators(compose_validators![
                required(),
                min_length(3),
                max_length(20),
            ]),
            email: FormControl::text("").with_validators(compose_validators![
                required(),
                email(),
            ]),
            password: FormControl::password("").with_validators(compose_validators![
                required(),
                min_length(8),
            ]),
            confirm_password: FormControl::password("").with_validator(
                match_field!("password")
            ),
            // Conditional validation
            company_name: FormControl::text("").with_validator(
                required_if!(is_company.get())
            ),
        };

        Self { form, is_company }
    }

    pub fn toggle_company_mode(&self) {
        self.is_company.update(|v| !v);
        // Re-validate conditional fields
        self.form.get_control("company_name")
            .unwrap()
            .update_validity();
    }
}

// ============================================================================
// COMPONENT WITH ALL MACROS
// ============================================================================

#[component(
    selector = "macro-showcase",
    template = html!(r#"
        <div class="showcase">
            <h1>Ferric Macros Showcase</h1>
            <div class="counter">
                <span>Count: {{ count }}</span>
                <button (click)="increment()">Increment</button>
            </div>
            <div class="form">
                <h2>Login Form</h2>
                <input type="email" [formControl]="email" />
                <input type="password" [formControl]="password" />
                <button [disabled]="!isValid()">Login</button>
            </div>
        </div>
    "#),
    styles = css!(r#"
        .showcase {
            padding: 20px;
            max-width: 800px;
            margin: 0 auto;
        }
        .counter {
            margin: 20px 0;
            padding: 15px;
            background: #f0f0f0;
            border-radius: 8px;
        }
        .form {
            margin: 20px 0;
        }
        input {
            display: block;
            margin: 10px 0;
            padding: 8px;
            width: 100%;
        }
        button {
            padding: 10px 20px;
            background: #007bff;
            color: white;
            border: none;
            border-radius: 4px;
            cursor: pointer;
        }
        button:disabled {
            background: #ccc;
            cursor: not-allowed;
        }
    "#)
)]
pub struct ShowcaseComponent {
    count: Signal<i32>,
    doubled: Computed<i32>,
    form: FormGroup,
}

impl ShowcaseComponent {
    pub fn new() -> Self {
        let count = signal!(0);
        let doubled = computed!(|| count.get() * 2);

        // Effect to log changes
        effect!(|| {
            web_sys::console::log_1(&format!("Count changed: {}", count.get()).into());
        });

        // Build form with validators
        let form = build_form! {
            email: FormControl::text("").with_validators(compose_validators![
                required(),
                email(),
            ]),
            password: FormControl::password("").with_validators(compose_validators![
                required(),
                min_length(8),
            ]),
        };

        Self {
            count,
            doubled,
            form,
        }
    }

    pub fn increment(&self) {
        self.count.update(|n| n + 1);
    }

    pub fn is_valid(&self) -> bool {
        self.form.is_valid()
    }
}

// ============================================================================
// ENTRY POINT
// ============================================================================

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    web_sys::console::log_1(&"=== Ferric Macros Showcase ===".into());

    web_sys::console::log_1(&"\n--- Reactive Macros ---".into());
    reactive_showcase();

    web_sys::console::log_1(&"\n--- Template Macros ---".into());
    template_showcase();

    web_sys::console::log_1(&"\n--- Forms Macros ---".into());
    forms_showcase();

    web_sys::console::log_1(&"\n--- Reactive Forms ---".into());
    reactive_forms_showcase();

    web_sys::console::log_1(&"\n=== Showcase Complete ===".into());
}

