//! User profile components.

use ferric_ssr::prelude::*;
use serde::{Deserialize, Serialize};

/// User data structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserData {
    pub username: String,
    pub display_name: String,
    pub email: String,
    pub bio: String,
}

/// User profile card component.
pub struct UserCard {
    user: UserData,
}

impl UserCard {
    /// Create a new user card.
    pub fn new(
        username: impl Into<String>,
        display_name: impl Into<String>,
        email: impl Into<String>,
        bio: impl Into<String>,
    ) -> Self {
        Self {
            user: UserData {
                username: username.into(),
                display_name: display_name.into(),
                email: email.into(),
                bio: bio.into(),
            },
        }
    }

    /// Create from user data.
    pub fn from_data(user: UserData) -> Self {
        Self { user }
    }
}

impl Renderable for UserCard {
    fn render(&self, ctx: &mut RenderContext) -> SsrResult<String> {
        ctx.enter()?;

        // Add state for hydration
        ctx.add_state("user", &self.user)?;

        let mut html = HtmlRenderer::new();

        html.open_tag("div")
            .attr("class", "user-card")
            .attr(
                "style",
                "text-align: center; padding: 2rem;",
            )
            .close_open();

        // Avatar
        html.open_tag("div")
            .attr("class", "avatar")
            .attr(
                "style",
                "width: 120px; height: 120px; border-radius: 50%; \
                 background: linear-gradient(135deg, #667eea, #764ba2); \
                 margin: 0 auto 1.5rem; display: flex; align-items: center; \
                 justify-content: center; font-size: 3rem; color: white;",
            )
            .close_open();
        // First letter of display name
        let initial = self
            .user
            .display_name
            .chars()
            .next()
            .unwrap_or('?')
            .to_uppercase()
            .to_string();
        html.text(&initial);
        html.close_tag("div");

        // Display name
        html.element(
            "h1",
            &[("style", "margin-bottom: 0.25rem;")],
            &self.user.display_name,
        );

        // Username
        html.element(
            "p",
            &[("style", "color: #667eea; font-weight: 500; margin-bottom: 1rem;")],
            &format!("@{}", self.user.username),
        );

        // Bio
        html.element(
            "p",
            &[("style", "color: #666; margin-bottom: 1.5rem;")],
            &self.user.bio,
        );

        // Email
        html.open_tag("div")
            .attr(
                "style",
                "display: inline-flex; align-items: center; gap: 0.5rem; \
                 background: #f8f9fa; padding: 0.5rem 1rem; border-radius: 20px;",
            )
            .close_open();
        html.text("📧 ");
        html.element(
            "a",
            &[
                ("href", &format!("mailto:{}", self.user.email)),
                ("style", "color: #667eea; text-decoration: none;"),
            ],
            &self.user.email,
        );
        html.close_tag("div");

        // Stats section
        html.open_tag("div")
            .attr(
                "style",
                "display: flex; justify-content: center; gap: 2rem; \
                 margin-top: 2rem; padding-top: 2rem; border-top: 1px solid #eee;",
            )
            .close_open();

        // Stat items
        let stats = [
            ("Posts", "42"),
            ("Followers", "1.2k"),
            ("Following", "256"),
        ];

        for (label, value) in stats {
            html.open_tag("div")
                .attr("style", "text-align: center;")
                .close_open();
            html.element(
                "div",
                &[("style", "font-size: 1.5rem; font-weight: bold; color: #1a1a2e;")],
                value,
            );
            html.element("div", &[("style", "color: #888; font-size: 0.9rem;")], label);
            html.close_tag("div");
        }

        html.close_tag("div");

        // Action buttons
        html.open_tag("div")
            .attr(
                "style",
                "display: flex; justify-content: center; gap: 1rem; margin-top: 1.5rem;",
            )
            .close_open();

        html.open_tag("button")
            .attr(
                "style",
                "padding: 0.75rem 2rem; background: #667eea; color: white; \
                 border: none; border-radius: 8px; cursor: pointer; font-weight: bold;",
            )
            .close_open();
        html.text("Follow");
        html.close_tag("button");

        html.open_tag("button")
            .attr(
                "style",
                "padding: 0.75rem 2rem; background: white; color: #667eea; \
                 border: 2px solid #667eea; border-radius: 8px; cursor: pointer; font-weight: bold;",
            )
            .close_open();
        html.text("Message");
        html.close_tag("button");

        html.close_tag("div");

        html.close_tag("div");

        ctx.exit();
        Ok(html.finish())
    }
}

