//! Component templates

/// Generate component Rust file
pub fn component_rs(name: &str, kebab_name: &str, inline_template: bool, inline_style: bool) -> String {
    let template_part = if inline_template {
        format!(
            r##"    const TEMPLATE: &'static str = r#"
<div class="{kebab_name}">
    <h2>{name}</h2>
    <p>Component works!</p>
</div>
"#;"##,
            name = name,
            kebab_name = kebab_name
        )
    } else {
        format!(
            r#"    const TEMPLATE: &'static str = include_str!("../../../templates/components/{kebab_name}.html");"#,
            kebab_name = kebab_name
        )
    };

    let style_part = if inline_style {
        format!(
            r##"    const STYLES: &'static str = r#"
.{kebab_name} {{
    display: block;
    padding: 1rem;
}}
"#;"##,
            kebab_name = kebab_name
        )
    } else {
        format!(
            r#"    const STYLES: &'static str = include_str!("../../styles/components/{kebab_name}.scss");"#,
            kebab_name = kebab_name
        )
    };

    format!(
        r#"//! {name} component

use ferric_core::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::{{Document, Element}};

/// {name} component
pub struct {name} {{
    root: Element,
}}

impl {name} {{
{template_part}

{style_part}

    /// Create a new {name} component
    pub fn new(document: &Document, parent: &Element) -> Result<Self, JsValue> {{
        let root = document.create_element("div")?;
        root.set_class_name("{kebab_name}");
        root.set_inner_html(Self::TEMPLATE);
        parent.append_child(&root)?;

        // Inject styles
        Self::inject_styles(document)?;

        Ok(Self {{ root }})
    }}

    fn inject_styles(document: &Document) -> Result<(), JsValue> {{
        let style = document.create_element("style")?;
        style.set_text_content(Some(Self::STYLES));
        if let Some(head) = document.head() {{
            head.append_child(&style)?;
        }}
        Ok(())
    }}

    /// Get the root element
    pub fn element(&self) -> &Element {{
        &self.root
    }}
}}

impl Drop for {name} {{
    fn drop(&mut self) {{
        let _ = self.root.remove();
    }}
}}
"#,
        name = name,
        kebab_name = kebab_name,
        template_part = template_part,
        style_part = style_part
    )
}

/// Generate component mod.rs
pub fn mod_rs(snake_name: &str, pascal_name: &str) -> String {
    format!(
        r#"//! {pascal_name} component module

mod {snake_name};

pub use {snake_name}::{pascal_name};
"#,
        pascal_name = pascal_name,
        snake_name = snake_name
    )
}

/// Generate component HTML template
pub fn template(name: &str) -> String {
    format!(
        r#"<div class="component-content">
    <h2>{name}</h2>
    <p>Component works!</p>
</div>
"#,
        name = name
    )
}

/// Generate component SCSS styles
pub fn styles(kebab_name: &str) -> String {
    format!(
        r#".{kebab_name} {{
    display: block;
    padding: 1rem;

    h2 {{
        margin: 0 0 0.5rem;
        font-size: 1.25rem;
        color: #333;
    }}

    p {{
        margin: 0;
        color: #666;
    }}
}}
"#,
        kebab_name = kebab_name
    )
}
