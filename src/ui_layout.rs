//! Bounded UI sizing helpers shared by the seven OS-native presentations.
//!
//! The viewport minimum is 560 × 400, but the usable inner rectangle can be
//! smaller once panels and scrollbars are accounted for. Never grow a field or
//! modal merely because the requested preferred width does not fit.

/// Width of a centered dialog, always clamped to the usable viewport.
pub(crate) fn dialog_width(viewport_width: f32, preferred: f32) -> f32 {
    (viewport_width - 64.0).max(0.0).min(preferred)
}

/// Width of a text field next to a browse button in a wrapping toolbar.
/// The caller must allow wrapping when the remaining width is exceptionally small.
pub(crate) fn field_width(available_width: f32, button_reserve: f32, preferred: f32) -> f32 {
    (available_width - button_reserve).max(0.0).min(preferred)
}

/// The content pane is only split when both children and their spacing fit.
pub(crate) fn split_panes(available_width: f32, list_min: f32, inspector_min: f32) -> bool {
    available_width >= list_min + inspector_min + 24.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modal_and_input_never_exceed_available_space() {
        for width in [280.0, 400.0, 560.0, 800.0, 1080.0, 1440.0] {
            let modal = dialog_width(width, 460.0);
            assert!(modal <= width);
            assert!(modal >= 0.0);
            assert!(field_width(modal, 112.0, 340.0) <= modal);
            assert!(field_width(modal, 112.0, 340.0) >= 0.0);
        }
    }

    #[test]
    fn inspector_only_appears_when_both_panes_fit() {
        assert!(!split_panes(560.0, 330.0, 235.0));
        assert!(!split_panes(580.0, 330.0, 235.0));
        assert!(split_panes(800.0, 330.0, 235.0));
    }
}
