use crate::{Button, ComboBox, InnerResponse, Response};

/// Dark or Light theme.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum Theme {
    /// Dark mode: light text on a dark background.
    Dark,

    /// Light mode: dark text on a light background.
    Light,
}

impl Theme {
    /// Get the egui visuals corresponding to this theme.
    ///
    /// Use with [`egui::Context::set_visuals`].
    #[deprecated]
    pub fn egui_visuals(self) -> crate::Visuals {
        match self {
            Self::Dark => crate::Visuals::dark(),
            Self::Light => crate::Visuals::light(),
        }
    }
}

/// The user's theme preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum ThemePreference {
    /// Dark mode: light text on a dark background.
    Dark,

    /// Light mode: dark text on a light background.
    Light,

    /// Follow the system's theme preference.
    System,
}

impl From<Theme> for ThemePreference {
    fn from(value: Theme) -> Self {
        match value {
            Theme::Dark => ThemePreference::Dark,
            Theme::Light => ThemePreference::Light,
        }
    }
}

impl ThemePreference {
    /// Shows a combo box to switch between dark, light and follow system.
    pub fn small_combo_box(&mut self, ui: &mut crate::Ui) -> Response {
        let InnerResponse {
            inner: changed,
            mut response,
        } = ComboBox::from_id_source("Theme Preference")
            .selected_text(icon(*self))
            .width(0.0)
            .show_ui(ui, |ui| self.radio_buttons_impl(ui));
        if changed.unwrap_or_default() {
            response.mark_changed();
        }
        response
    }

    /// Shows radio buttons to switch between dark, light and follow system.
    pub fn radio_buttons(&mut self, ui: &mut crate::Ui) -> Response {
        let InnerResponse {
            inner: changed,
            mut response,
        } = ui.horizontal(|ui| self.radio_buttons_impl(ui));
        if changed {
            response.mark_changed();
        }
        response
    }

    fn radio_buttons_impl(&mut self, ui: &mut crate::Ui) -> bool {
        let mut changed = false;
        changed |= ui
            .selectable_value(self, Self::System, label(Self::System))
            .changed();
        changed |= ui
            .selectable_value(self, Self::Dark, label(Self::Dark))
            .changed();
        changed |= ui
            .selectable_value(self, Self::Light, label(Self::Light))
            .changed();
        changed
    }
}

fn icon(preference: ThemePreference) -> &'static str {
    match preference {
        ThemePreference::Dark => "🌙",
        ThemePreference::Light => "☀",
        ThemePreference::System => "✨",
    }
}

fn label(preference: ThemePreference) -> &'static str {
    match preference {
        ThemePreference::Dark => "🌙 Dark",
        ThemePreference::Light => "☀ Light",
        ThemePreference::System => "✨ Follow System",
    }
}
