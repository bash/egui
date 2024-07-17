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
