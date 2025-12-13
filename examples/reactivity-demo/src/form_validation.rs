//! # Form Validation Demo
//!
//! Demonstrates:
//! - Complex computed validation state
//! - Real-time error messages
//! - Submit state management

use ferric_core::reactive::{batch, computed, effect, signal, Computed, Effect, Signal};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlInputElement};

// Load template and styles at compile time
const TEMPLATE: &str = include_str!("../templates/form_validation.html");
const STYLES: &str = include_str!("../styles/form_validation.css");

#[derive(Debug, Clone, Default)]
pub struct FieldState {
    pub value: String,
    pub touched: bool,
    pub dirty: bool,
}

pub struct FormValidationDemo {
    username: Signal<FieldState>,
    email: Signal<FieldState>,
    password: Signal<FieldState>,
    confirm_password: Signal<FieldState>,
    username_errors: Computed<Vec<String>>,
    email_errors: Computed<Vec<String>>,
    password_errors: Computed<Vec<String>>,
    confirm_errors: Computed<Vec<String>>,
    is_valid: Computed<bool>,
    is_submitting: Signal<bool>,
    submit_result: Signal<Option<Result<String, String>>>,
    _effects: Vec<Effect>,
}

impl FormValidationDemo {
    fn validate_username(state: &FieldState) -> Vec<String> {
        let mut errors = vec![];
        let value = state.value.trim();

        if value.is_empty() {
            errors.push("Username is required".to_string());
        } else {
            if value.len() < 3 {
                errors.push("Username must be at least 3 characters".to_string());
            }
            if value.len() > 20 {
                errors.push("Username must be at most 20 characters".to_string());
            }
            if !value.chars().all(|c| c.is_alphanumeric() || c == '_') {
                errors.push("Username can only contain letters, numbers, and underscores".to_string());
            }
        }
        errors
    }

    fn validate_email(state: &FieldState) -> Vec<String> {
        let mut errors = vec![];
        let value = state.value.trim();

        if value.is_empty() {
            errors.push("Email is required".to_string());
        } else if !value.contains('@') || !value.contains('.') {
            errors.push("Please enter a valid email address".to_string());
        }
        errors
    }

    fn validate_password(state: &FieldState) -> Vec<String> {
        let mut errors = vec![];
        let value = &state.value;

        if value.is_empty() {
            errors.push("Password is required".to_string());
        } else {
            if value.len() < 8 {
                errors.push("Password must be at least 8 characters".to_string());
            }
            if !value.chars().any(|c| c.is_uppercase()) {
                errors.push("Password must contain an uppercase letter".to_string());
            }
            if !value.chars().any(|c| c.is_lowercase()) {
                errors.push("Password must contain a lowercase letter".to_string());
            }
            if !value.chars().any(|c| c.is_numeric()) {
                errors.push("Password must contain a number".to_string());
            }
            if !value.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:',.<>?".contains(c)) {
                errors.push("Password must contain a special character".to_string());
            }
        }
        errors
    }

    fn validate_confirm(password: &str, confirm: &FieldState) -> Vec<String> {
        let mut errors = vec![];

        if confirm.value.is_empty() {
            errors.push("Please confirm your password".to_string());
        } else if confirm.value != password {
            errors.push("Passwords do not match".to_string());
        }
        errors
    }

    pub fn mount(container: &Element) -> Result<(), JsValue> {
        inject_styles("form-validation-styles", STYLES);

        let username = signal(FieldState::default());
        let email = signal(FieldState::default());
        let password = signal(FieldState::default());
        let confirm_password = signal(FieldState::default());

        let username_for_valid = username.clone();
        let username_errors = computed(move || {
            let state = username_for_valid.get();
            if !state.touched { return vec![]; }
            Self::validate_username(&state)
        });

        let email_for_valid = email.clone();
        let email_errors = computed(move || {
            let state = email_for_valid.get();
            if !state.touched { return vec![]; }
            Self::validate_email(&state)
        });

        let password_for_valid = password.clone();
        let password_errors = computed(move || {
            let state = password_for_valid.get();
            if !state.touched { return vec![]; }
            Self::validate_password(&state)
        });

        let password_for_confirm = password.clone();
        let confirm_for_valid = confirm_password.clone();
        let confirm_errors = computed(move || {
            let confirm_state = confirm_for_valid.get();
            if !confirm_state.touched { return vec![]; }
            let pwd_state = password_for_confirm.get();
            Self::validate_confirm(&pwd_state.value, &confirm_state)
        });

        let username_for_form = username.clone();
        let email_for_form = email.clone();
        let password_for_form = password.clone();
        let confirm_for_form = confirm_password.clone();
        let is_valid = computed(move || {
            let u = Self::validate_username(&username_for_form.get());
            let e = Self::validate_email(&email_for_form.get());
            let p = Self::validate_password(&password_for_form.get());
            let c = Self::validate_confirm(&password_for_form.get().value, &confirm_for_form.get());
            u.is_empty() && e.is_empty() && p.is_empty() && c.is_empty()
        });

        let is_submitting = signal(false);
        let submit_result: Signal<Option<Result<String, String>>> = signal(None);

        // Render template
        container.set_inner_html(TEMPLATE);

        Self::setup_input_handler(container, "#username", username.clone());
        Self::setup_input_handler(container, "#email", email.clone());
        Self::setup_input_handler(container, "#password", password.clone());
        Self::setup_input_handler(container, "#confirm-password", confirm_password.clone());

        // Error display effects
        let username_errors_for_ui = username_errors.clone();
        let container_for_u = container.clone();
        let username_error_effect = effect(move || {
            Self::display_errors(&container_for_u, "#username-errors", "#username", &username_errors_for_ui.get());
        });

        let email_errors_for_ui = email_errors.clone();
        let container_for_e = container.clone();
        let email_error_effect = effect(move || {
            Self::display_errors(&container_for_e, "#email-errors", "#email", &email_errors_for_ui.get());
        });

        let password_errors_for_ui = password_errors.clone();
        let container_for_p = container.clone();
        let password_error_effect = effect(move || {
            Self::display_errors(&container_for_p, "#password-errors", "#password", &password_errors_for_ui.get());
        });

        let confirm_errors_for_ui = confirm_errors.clone();
        let container_for_c = container.clone();
        let confirm_error_effect = effect(move || {
            Self::display_errors(&container_for_c, "#confirm-errors", "#confirm-password", &confirm_errors_for_ui.get());
        });

        // Password strength indicator
        let password_for_strength = password.clone();
        let container_for_strength = container.clone();
        let strength_effect = effect(move || {
            let state = password_for_strength.get();
            let pwd = &state.value;

            let mut score = 0;
            if pwd.len() >= 8 { score += 1; }
            if pwd.len() >= 12 { score += 1; }
            if pwd.chars().any(|c| c.is_uppercase()) { score += 1; }
            if pwd.chars().any(|c| c.is_lowercase()) { score += 1; }
            if pwd.chars().any(|c| c.is_numeric()) { score += 1; }
            if pwd.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:',.<>?".contains(c)) { score += 1; }

            let (width, color, text) = match score {
                0 => ("0%", "#374151", "Enter a password"),
                1 => ("16%", "#ef4444", "Very Weak"),
                2 => ("33%", "#f97316", "Weak"),
                3 => ("50%", "#eab308", "Fair"),
                4 => ("66%", "#84cc16", "Good"),
                5 => ("83%", "#22c55e", "Strong"),
                _ => ("100%", "#10b981", "Very Strong"),
            };

            if let Some(fill) = container_for_strength.query_selector("#strength-fill").ok().flatten() {
                let _ = fill.set_attribute("style", &format!("width: {}; background: {}", width, color));
            }
            if let Some(text_el) = container_for_strength.query_selector("#strength-text").ok().flatten() {
                text_el.set_text_content(Some(text));
            }
        });

        // Submit button state
        let is_valid_for_btn = is_valid.clone();
        let is_submitting_for_btn = is_submitting.clone();
        let container_for_btn = container.clone();
        let submit_btn_effect = effect(move || {
            let valid = is_valid_for_btn.get();
            let submitting = is_submitting_for_btn.get();

            if let Some(btn) = container_for_btn.query_selector("#submit-btn").ok().flatten() {
                if submitting {
                    let _ = btn.set_attribute("disabled", "");
                    btn.set_text_content(Some("Creating..."));
                } else if valid {
                    let _ = btn.remove_attribute("disabled");
                    btn.set_text_content(Some("Create Account"));
                } else {
                    let _ = btn.set_attribute("disabled", "");
                    btn.set_text_content(Some("Create Account"));
                }
            }
        });

        // Submit result display
        let submit_result_for_ui = submit_result.clone();
        let container_for_result = container.clone();
        let result_effect = effect(move || {
            let result = submit_result_for_ui.get();
            if let Some(el) = container_for_result.query_selector("#submit-result").ok().flatten() {
                match result {
                    Some(Ok(msg)) => {
                        el.set_text_content(Some(&msg));
                        let _ = el.set_attribute("class", "submit-result success");
                    }
                    Some(Err(msg)) => {
                        el.set_text_content(Some(&msg));
                        let _ = el.set_attribute("class", "submit-result error");
                    }
                    None => {
                        let _ = el.set_attribute("class", "submit-result hidden");
                    }
                }
            }
        });

        // Debug state display
        let username_for_debug = username.clone();
        let email_for_debug = email.clone();
        let password_for_debug = password.clone();
        let confirm_for_debug = confirm_password.clone();
        let is_valid_for_debug = is_valid.clone();
        let container_for_debug = container.clone();
        let debug_effect = effect(move || {
            let u = username_for_debug.get();
            let e = email_for_debug.get();
            let p = password_for_debug.get();
            let c = confirm_for_debug.get();
            let valid = is_valid_for_debug.get();

            let debug_text = format!(
                "username: {{ value: \"{}\", touched: {}, dirty: {} }}\n\
                 email: {{ value: \"{}\", touched: {}, dirty: {} }}\n\
                 password: {{ value: \"***\", touched: {}, dirty: {} }}\n\
                 confirm: {{ value: \"***\", touched: {}, dirty: {} }}\n\
                 isValid: {}",
                u.value, u.touched, u.dirty,
                e.value, e.touched, e.dirty,
                p.touched, p.dirty,
                c.touched, c.dirty,
                valid
            );

            if let Some(el) = container_for_debug.query_selector("#form-state-debug").ok().flatten() {
                el.set_text_content(Some(&debug_text));
            }
        });

        // Form submit handler
        if let Some(form) = container.query_selector("#signup-form").ok().flatten() {
            let is_valid = is_valid.clone();
            let is_submitting = is_submitting.clone();
            let submit_result = submit_result.clone();
            let username = username.clone();
            let email = email.clone();

            let closure = Closure::wrap(Box::new(move |e: web_sys::Event| {
                e.prevent_default();

                if !is_valid.get() { return; }

                is_submitting.set(true);
                submit_result.set(None);

                let is_submitting = is_submitting.clone();
                let submit_result = submit_result.clone();
                let username_value = username.get().value.clone();
                let email_value = email.get().value.clone();

                let closure = Closure::once(Box::new(move || {
                    is_submitting.set(false);
                    submit_result.set(Some(Ok(format!(
                        "✅ Account created for {} ({})!",
                        username_value, email_value
                    ))));
                }) as Box<dyn FnOnce()>);

                if let Some(window) = web_sys::window() {
                    let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                        closure.as_ref().unchecked_ref(),
                        1500,
                    );
                }
                closure.forget();
            }) as Box<dyn Fn(_)>);

            let _ = form.add_event_listener_with_callback("submit", closure.as_ref().unchecked_ref());
            closure.forget();
        }

        // Reset button handler
        if let Some(btn) = container.query_selector("#reset-btn").ok().flatten() {
            let username = username.clone();
            let email = email.clone();
            let password = password.clone();
            let confirm_password = confirm_password.clone();
            let submit_result = submit_result.clone();
            let container = container.clone();

            let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                batch(|| {
                    username.set(FieldState::default());
                    email.set(FieldState::default());
                    password.set(FieldState::default());
                    confirm_password.set(FieldState::default());
                    submit_result.set(None);
                });

                for selector in ["#username", "#email", "#password", "#confirm-password"] {
                    if let Some(input) = container.query_selector(selector).ok().flatten() {
                        if let Ok(input) = input.dyn_into::<HtmlInputElement>() {
                            input.set_value("");
                        }
                    }
                }
            }) as Box<dyn Fn(_)>);

            let _ = btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref());
            closure.forget();
        }

        std::mem::forget(FormValidationDemo {
            username,
            email,
            password,
            confirm_password,
            username_errors,
            email_errors,
            password_errors,
            confirm_errors,
            is_valid,
            is_submitting,
            submit_result,
            _effects: vec![
                username_error_effect,
                email_error_effect,
                password_error_effect,
                confirm_error_effect,
                strength_effect,
                submit_btn_effect,
                result_effect,
                debug_effect,
            ],
        });

        Ok(())
    }

    fn setup_input_handler(container: &Element, selector: &str, field: Signal<FieldState>) {
        if let Some(input) = container.query_selector(selector).ok().flatten() {
            let field_for_input = field.clone();
            let input_closure = Closure::wrap(Box::new(move |e: web_sys::Event| {
                if let Some(target) = e.target() {
                    if let Ok(input) = target.dyn_into::<HtmlInputElement>() {
                        field_for_input.mutate(|state| {
                            state.value = input.value();
                            state.dirty = true;
                        });
                    }
                }
            }) as Box<dyn Fn(_)>);
            let _ = input.add_event_listener_with_callback("input", input_closure.as_ref().unchecked_ref());
            input_closure.forget();

            let field_for_blur = field.clone();
            let blur_closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                field_for_blur.mutate(|state| {
                    state.touched = true;
                });
            }) as Box<dyn Fn(_)>);
            let _ = input.add_event_listener_with_callback("blur", blur_closure.as_ref().unchecked_ref());
            blur_closure.forget();
        }
    }

    fn display_errors(container: &Element, error_selector: &str, input_selector: &str, errors: &[String]) {
        if let Some(el) = container.query_selector(error_selector).ok().flatten() {
            if errors.is_empty() {
                el.set_inner_html("");
            } else {
                let html: String = errors.iter()
                    .map(|e| format!(r#"<div class="error-item">⚠️ {}</div>"#, e))
                    .collect();
                el.set_inner_html(&html);
            }
        }

        if let Some(input) = container.query_selector(input_selector).ok().flatten() {
            let class_list = input.class_list();
            if errors.is_empty() {
                let _ = class_list.remove_1("invalid");
            } else {
                let _ = class_list.add_1("invalid");
            }
        }
    }
}

fn inject_styles(id: &str, css: &str) {
    if let Some(document) = web_sys::window().and_then(|w| w.document()) {
        if document.get_element_by_id(id).is_some() {
            return;
        }
        if let Ok(style) = document.create_element("style") {
            let _ = style.set_attribute("id", id);
            style.set_text_content(Some(css));
            if let Some(head) = document.head() {
                let _ = head.append_child(&style);
            }
        }
    }
}
