# Ferric Forms Macros Guide

Comprehensive guide to form-related macros in Ferric.

## Table of Contents

1. [Basic Form Macros](#basic-form-macros)
2. [Validation Macros](#validation-macros)
3. [Type-Safe Forms](#type-safe-forms)
4. [Form DSL](#form-dsl)
5. [Advanced Examples](#advanced-examples)

## Basic Form Macros

### `form_group!`

Create a form group declaratively:

```rust
let form = form_group! {
    name: FormControl::text(""),
    email: FormControl::text(""),
    age: FormControl::number(0),
    address: form_group! {
        street: FormControl::text(""),
        city: FormControl::text(""),
        zip: FormControl::text(""),
    },
};
```

### `form_array!`

Create a form array:

```rust
let phones = form_array![
    FormControl::text("555-1234"),
    FormControl::text("555-5678"),
    FormControl::text("555-9012"),
];
```

### `form_control!`

Create a form control with validators:

```rust
let email = form_control!(String, "", [required(), email()]);
let age = form_control!(i32, 0, [required(), min(18), max(120)]);
```

### `validators!`

Compose validators:

```rust
let validators = validators![
    required(),
    min_length(2),
    max_length(50),
];
```

## Validation Macros

### `#[derive(Validate)]`

Auto-generate validation logic:

```rust
#[derive(Validate)]
struct SignupForm {
    #[validate(required, email)]
    email: String,

    #[validate(required, min_length = 8, custom = "validate_password_strength")]
    password: String,

    #[validate(required, min = 18, max = 120)]
    age: i32,

    #[validate(url)]
    website: Option<String>,
}

fn validate_password_strength(password: &str) -> Result<(), String> {
    if !password.chars().any(|c| c.is_uppercase()) {
        return Err("Password must contain uppercase letter".to_string());
    }
    Ok(())
}
```

### `compose_validators!`

Compose validators into a vector:

```rust
let validators = compose_validators![
    required(),
    min_length(3),
    max_length(50),
    pattern(r"^[a-zA-Z0-9_]+$"),
];

control.set_validators(validators);
```

### `required_if!`

Conditional required validator:

```rust
let phone_required = required_if!(has_mobile_access.get());
let ssn_required = required_if!(is_us_citizen.get() && age.get() >= 18);
```

### `match_field!`

Cross-field matching validator:

```rust
// Password confirmation must match password
let password_confirm = FormControl::password("")
    .with_validator(match_field!("password"));
```

### `async_validator!`

Create an async validator:

```rust
let username_validator = async_validator!(|value: String| async move {
    let available = check_username_availability(&value).await?;
    if available {
        Ok(())
    } else {
        Err("Username already taken".to_string())
    }
});
```

## Type-Safe Forms

### `#[derive(TypedForm)]`

Generate a type-safe form from a struct:

```rust
#[derive(TypedForm)]
struct User {
    name: String,
    email: String,
    age: u32,
}

// Generates UserForm with typed controls
let user = User {
    name: "Alice".to_string(),
    email: "alice@example.com".to_string(),
    age: 30,
};

let form = UserForm::from_value(user);

// Type-safe access
form.name.set_value("Bob".to_string());
form.email.set_value("bob@example.com".to_string());

// Extract back to User
let updated_user: User = form.into();
```

## Form DSL

### `build_form!`

Declarative form building with validation:

```rust
let form = build_form! {
    username: FormControl::text("").with_validators(compose_validators![
        required(),
        min_length(3),
        max_length(20),
        pattern(r"^[a-zA-Z0-9_]+$"),
    ]),

    email: FormControl::text("").with_validators(compose_validators![
        required(),
        email(),
    ]),

    password: FormControl::password("").with_validators(compose_validators![
        required(),
        min_length(8),
    ]),

    confirm_password: FormControl::password("").with_validator(match_field!("password")),

    age: FormControl::number(0).with_validators(compose_validators![
        required(),
        min(18),
        max(120),
    ]),
};
```

### `reactive_form!`

Create a reactive form with signal integration:

```rust
let form = reactive_form! {
    name: FormControl::text(""),
    email: FormControl::text(""),
    age: FormControl::number(0),
};

// form is a Signal<FormGroup>
effect!(|| {
    let form_value = form.get();
    if form_value.is_valid() {
        println!("Form is valid!");
        println!("Values: {:?}", form_value.value());
    } else {
        println!("Errors: {:?}", form_value.errors());
    }
});
```

## Advanced Examples

### Complete Login Form

```rust
use ferric_forms::prelude::*;
use ferric_core::prelude::*;

#[component(
    selector = "login-form",
    template = r#"
        <form (submit)="onSubmit()">
            <input type="email" [formControl]="email" />
            <input type="password" [formControl]="password" />
            <button [disabled]="!isValid()">Login</button>
        </form>
    "#
)]
pub struct LoginFormComponent {
    form: FormGroup,
}

impl LoginFormComponent {
    pub fn new() -> Self {
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

        Self { form }
    }

    pub fn is_valid(&self) -> bool {
        self.form.is_valid()
    }

    pub fn on_submit(&self) {
        if self.is_valid() {
            let email = self.form.get_value("email");
            let password = self.form.get_value("password");
            // Handle login...
        }
    }
}
```

### Registration Form with Type Safety

```rust
#[derive(TypedForm, Validate)]
struct RegistrationData {
    #[validate(required, min_length = 3, max_length = 50)]
    username: String,

    #[validate(required, email)]
    email: String,

    #[validate(required, min_length = 8)]
    password: String,

    #[validate(required, min = 18)]
    age: u32,

    #[validate(url)]
    website: Option<String>,
}

pub struct RegistrationForm {
    form: RegistrationDataForm,
}

impl RegistrationForm {
    pub fn new() -> Self {
        let data = RegistrationData {
            username: String::new(),
            email: String::new(),
            password: String::new(),
            age: 18,
            website: None,
        };

        Self {
            form: RegistrationDataForm::from_value(data),
        }
    }

    pub fn submit(&self) -> Result<RegistrationData, FormError> {
        if !self.form.is_valid() {
            return Err(FormError::ValidationFailed);
        }

        Ok(self.form.value())
    }
}
```

### Dynamic Form with Conditional Validation

```rust
pub struct DynamicForm {
    form: FormGroup,
    is_company: Signal<bool>,
}

impl DynamicForm {
    pub fn new() -> Self {
        let is_company = signal!(false);

        let form = reactive_form! {
            name: FormControl::text("").with_validators(compose_validators![
                required(),
            ]),
            email: FormControl::text("").with_validators(compose_validators![
                required(),
                email(),
            ]),
            // Company name is required if is_company is true
            company_name: FormControl::text("").with_validator(
                required_if!(is_company.get())
            ),
            tax_id: FormControl::text("").with_validator(
                required_if!(is_company.get())
            ),
        };

        // Watch for is_company changes
        let form_clone = form.clone();
        effect!(move || {
            if is_company.get() {
                println!("Company mode enabled");
                form_clone.get().get_control("company_name").unwrap().update_validity();
                form_clone.get().get_control("tax_id").unwrap().update_validity();
            }
        });

        Self { form, is_company }
    }

    pub fn toggle_company_mode(&self) {
        self.is_company.update(|v| !v);
    }
}
```

### Async Validation Example

```rust
pub struct UsernameForm {
    form: FormGroup,
}

impl UsernameForm {
    pub fn new(http_client: Arc<HttpClient>) -> Self {
        let username_validator = async_validator!(move |value: String| {
            let client = http_client.clone();
            async move {
                if value.len() < 3 {
                    return Err("Username too short".to_string());
                }

                let response = client
                    .get(&format!("/api/check-username/{}", value))
                    .send()
                    .await
                    .map_err(|e| e.to_string())?;

                let available: bool = response.json()?;

                if available {
                    Ok(())
                } else {
                    Err("Username already taken".to_string())
                }
            }
        });

        let form = build_form! {
            username: FormControl::text("").with_validators(vec![
                Box::new(required()),
                Box::new(username_validator),
            ]),
        };

        Self { form }
    }
}
```

## Best Practices

1. **Use `#[derive(TypedForm)]` for type safety**: Get compile-time guarantees
2. **Compose validators with `compose_validators!`**: Keep validation logic organized
3. **Use `build_form!` for complex forms**: More readable than manual construction
4. **Leverage `reactive_form!` with signals**: Automatic reactivity for form state
5. **Use `async_validator!` for server-side validation**: Check uniqueness, availability, etc.
6. **Apply `required_if!` for conditional fields**: Dynamic form requirements

## Performance Tips

- Use `reactive_form!` to automatically track form state changes
- Debounce async validators to reduce API calls
- Use `batch!` when updating multiple form controls
- Memoize complex validation logic with `memo!`
- Use `OnPush` change detection with form signals

## See Also

- [Ferric Macros Guide](../../ferric-macros/MACROS_GUIDE.md)
- [Forms Guide](../docs/FORMS.md)
- [Validation Guide](../docs/VALIDATION.md)

