//! Feature card component

/// Render a feature card
pub fn feature_card(icon: &str, title: &str, description: &str) -> String {
    format!(
        r#"<div class="group relative p-6 bg-white dark:bg-slate-800/50 rounded-2xl border border-slate-200 dark:border-slate-700 hover:border-orange-500/50 dark:hover:border-orange-500/50 transition-all hover:shadow-lg hover:shadow-orange-500/10">
            <div class="w-12 h-12 rounded-xl bg-gradient-to-br from-orange-500 to-red-600 flex items-center justify-center mb-4 shadow-lg shadow-orange-500/25 group-hover:scale-110 transition-transform">
                <span class="text-2xl">{icon}</span>
            </div>
            <h3 class="text-lg font-semibold text-slate-900 dark:text-white mb-2">{title}</h3>
            <p class="text-slate-600 dark:text-slate-400">{description}</p>
        </div>"#,
        icon = icon,
        title = title,
        description = description
    )
}

/// Render a stat card
pub fn stat_card(value: &str, label: &str) -> String {
    format!(
        r#"<div class="text-center">
            <div class="text-4xl font-bold bg-gradient-to-r from-orange-500 to-red-600 bg-clip-text text-transparent">{value}</div>
            <div class="text-sm text-slate-600 dark:text-slate-400 mt-1">{label}</div>
        </div>"#,
        value = value,
        label = label
    )
}


/// Render a feature card
pub fn feature_card(icon: &str, title: &str, description: &str) -> String {
    format!(
        r#"<div class="group relative p-6 bg-white dark:bg-slate-800/50 rounded-2xl border border-slate-200 dark:border-slate-700 hover:border-orange-500/50 dark:hover:border-orange-500/50 transition-all hover:shadow-lg hover:shadow-orange-500/10">
            <div class="w-12 h-12 rounded-xl bg-gradient-to-br from-orange-500 to-red-600 flex items-center justify-center mb-4 shadow-lg shadow-orange-500/25 group-hover:scale-110 transition-transform">
                <span class="text-2xl">{icon}</span>
            </div>
            <h3 class="text-lg font-semibold text-slate-900 dark:text-white mb-2">{title}</h3>
            <p class="text-slate-600 dark:text-slate-400">{description}</p>
        </div>"#,
        icon = icon,
        title = title,
        description = description
    )
}

/// Render a stat card
pub fn stat_card(value: &str, label: &str) -> String {
    format!(
        r#"<div class="text-center">
            <div class="text-4xl font-bold bg-gradient-to-r from-orange-500 to-red-600 bg-clip-text text-transparent">{value}</div>
            <div class="text-sm text-slate-600 dark:text-slate-400 mt-1">{label}</div>
        </div>"#,
        value = value,
        label = label
    )
}

