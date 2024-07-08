use emath::{Pos2, Rect};
use epaint::Color32;

use crate::{Painter, Response, ThemePreference, Ui, Widget};

mod arc;
mod cogwheel;
mod moon;
mod painter_ext;
mod sun;

#[must_use = "You should put this widget in an ui with `ui.add(widget);`"]
pub struct ThemeSwitch<'a> {
    value: &'a mut ThemePreference,
    show_follow_system: bool,
}

impl<'a> ThemeSwitch<'a> {
    pub fn new(value: &'a mut ThemePreference) -> Self {
        Self {
            value,
            show_follow_system: true,
        }
    }

    /// Disables the "Follow System" option. Intentionally internal.
    /// Should be removed once https://github.com/emilk/egui/issues/4490 is done.
    pub(crate) fn show_follow_system(mut self, show_follow_system: bool) -> Self {
        self.show_follow_system = show_follow_system;
        self
    }
}

impl<'a> Widget for ThemeSwitch<'a> {
    fn ui(self, ui: &mut crate::Ui) -> crate::Response {
        let (update, mut response) = switch(ui, *self.value, "Theme", self.options());

        // TODO: union inner responses
        if let Some(value) = update {
            response.mark_changed();
            *self.value = value;
        }

        response
    }
}

impl<'a> ThemeSwitch<'a> {
    fn options(&self) -> Vec<SwitchOption<ThemePreference>> {
        let system = SwitchOption {
            value: ThemePreference::System,
            icon: cogwheel::cogwheel,
            label: "Follow System",
        };
        let dark = SwitchOption {
            value: ThemePreference::Dark,
            icon: moon::moon,
            label: "Dark",
        };
        let light = SwitchOption {
            value: ThemePreference::Light,
            icon: sun::sun,
            label: "Light",
        };

        let mut options = Vec::with_capacity(3);
        if self.show_follow_system {
            options.push(system);
        }
        options.extend([dark, light]);
        options
    }
}

#[derive(Debug)]
struct SwitchOption<T> {
    value: T,
    icon: IconPainter,
    label: &'static str,
}

type IconPainter = fn(&Painter, Pos2, f32, Color32);

fn switch<T>(
    ui: &mut Ui,
    value: T,
    label: &str,
    options: Vec<SwitchOption<T>>,
) -> (Option<T>, Response)
where
    T: PartialEq + Clone,
{
    let mut space = space_allocation::allocate_space(ui, options);

    let updated_value = interactivity::update_value_on_click(&mut space, &value);
    let value = updated_value.clone().unwrap_or(value);

    if ui.is_rect_visible(space.rect) {
        painting::draw_switch_background(ui, &space);
        painting::draw_active_indicator(ui, &space, &value);

        for button in &space.buttons {
            painting::draw_button(ui, button, value == button.option.value);
        }
    }

    let response = accessibility::attach_widget_info(ui, space, label, &value);

    (updated_value, response)
}

struct AllocatedSpace<T> {
    response: Response,
    rect: Rect,
    buttons: Vec<ButtonSpace<T>>,
    radius: f32,
}

struct ButtonSpace<T> {
    center: Pos2,
    response: Response,
    radius: f32,
    option: SwitchOption<T>,
}

mod space_allocation {
    use super::*;
    use crate::{Id, Sense};
    use emath::vec2;

    pub(super) fn allocate_space<T>(
        ui: &mut Ui,
        options: Vec<SwitchOption<T>>,
    ) -> AllocatedSpace<T> {
        let (rect, response, measurements) = allocate_switch(ui, &options);
        let id = response.id;

        // Focusable elements always get an accessible node, so let's ensure that
        // the parent is set correctly when the responses are created the first time.
        with_accessibility_parent(ui, id, |ui| {
            let buttons = options
                .into_iter()
                .enumerate()
                .scan(rect, |remaining, (n, option)| {
                    Some(allocate_button(ui, remaining, id, &measurements, n, option))
                })
                .collect();

            AllocatedSpace {
                response,
                rect,
                buttons,
                radius: measurements.radius,
            }
        })
    }

    fn allocate_switch<T>(
        ui: &mut Ui,
        options: &[SwitchOption<T>],
    ) -> (Rect, Response, SwitchMeasurements) {
        let diameter = ui.spacing().interact_size.y;
        let radius = diameter / 2.0;
        let padding = ui.spacing().button_padding.min_elem();
        let min_gap = 0.5 * ui.spacing().item_spacing.x;
        let gap_count = options.len().saturating_sub(1) as f32;
        let button_count = options.len() as f32;

        let min_size = vec2(
            button_count * diameter + (gap_count * min_gap) + (2.0 * padding),
            diameter + (2.0 * padding),
        );
        let sense = Sense::focusable_noninteractive();
        let (rect, response) = ui.allocate_at_least(min_size, sense);

        // The space we're given might be larger so we calculate
        // the margin based on the allocated rect.
        let total_gap = rect.width() - (button_count * diameter) - (2.0 * padding);
        let gap = total_gap / gap_count;

        let measurements = SwitchMeasurements {
            gap,
            radius,
            padding,
            buttons: options.len(),
        };

        (rect, response, measurements)
    }

    struct SwitchMeasurements {
        gap: f32,
        radius: f32,
        padding: f32,
        buttons: usize,
    }

    fn with_accessibility_parent<T>(ui: &mut Ui, id: Id, f: impl FnOnce(&mut Ui) -> T) -> T {
        let ctx = ui.ctx().clone();
        let mut result = None;
        ctx.with_accessibility_parent(id, || result = Some(f(ui)));
        result.expect("with_accessibility_parent() always calls f()")
    }

    fn allocate_button<T>(
        ui: &mut Ui,
        remaining: &mut Rect,
        switch_id: Id,
        measurements: &SwitchMeasurements,
        n: usize,
        option: SwitchOption<T>,
    ) -> ButtonSpace<T> {
        let (rect, center) = partition(remaining, measurements, n);
        let response = ui.interact(rect, switch_id.with(n), Sense::click());
        ButtonSpace {
            center,
            response,
            radius: measurements.radius,
            option,
        }
    }

    fn partition(
        remaining: &mut Rect,
        measurements: &SwitchMeasurements,
        n: usize,
    ) -> (Rect, Pos2) {
        let (leading, trailing) = offset(n, measurements);
        let center = remaining.left_center() + vec2(leading + measurements.radius, 0.0);
        let right = remaining.min.x + leading + 2.0 * measurements.radius + trailing;
        let (rect, new_remaining) = remaining.split_left_right_at_x(right);
        *remaining = new_remaining;
        (rect, center)
    }

    // Calculates the leading and trailing space for a button.
    // The gap between buttons is divided up evenly so that the entire
    // switch is clickable.
    fn offset(n: usize, measurements: &SwitchMeasurements) -> (f32, f32) {
        let leading = if n == 0 {
            measurements.padding
        } else {
            measurements.gap / 2.0
        };
        let trailing = if n == measurements.buttons - 1 {
            measurements.padding
        } else {
            measurements.gap / 2.0
        };
        (leading, trailing)
    }
}

mod interactivity {
    use super::*;

    pub(super) fn update_value_on_click<T>(space: &mut AllocatedSpace<T>, value: &T) -> Option<T>
    where
        T: PartialEq + Clone,
    {
        let clicked = space
            .buttons
            .iter_mut()
            .find(|b| b.response.clicked())
            .filter(|b| &b.option.value != value)?;
        clicked.response.mark_changed();
        Some(clicked.option.value.clone())
    }
}

mod painting {
    use super::*;
    use crate::style::WidgetVisuals;
    use crate::Id;
    use emath::pos2;
    use epaint::Stroke;

    pub(super) fn draw_switch_background<T>(ui: &mut Ui, space: &AllocatedSpace<T>) {
        let rect = space.rect;
        let rounding = 0.5 * rect.height();
        let WidgetVisuals {
            bg_fill, bg_stroke, ..
        } = switch_visuals(ui, &space.response);
        ui.painter().rect(rect, rounding, bg_fill, bg_stroke);
    }

    fn switch_visuals(ui: &Ui, response: &Response) -> WidgetVisuals {
        if response.has_focus() {
            ui.style().visuals.widgets.hovered
        } else {
            ui.style().visuals.widgets.inactive
        }
    }

    pub(super) fn draw_active_indicator<T: PartialEq>(
        ui: &mut Ui,
        space: &AllocatedSpace<T>,
        value: &T,
    ) {
        let fill = ui.visuals().selection.bg_fill;
        if let Some(pos) = space
            .buttons
            .iter()
            .find(|button| &button.option.value == value)
            .map(|button| button.center)
        {
            let pos = animate_active_indicator_position(ui, space.response.id, pos);
            ui.painter().circle(pos, space.radius, fill, Stroke::NONE);
        }
    }

    fn animate_active_indicator_position(ui: &mut Ui, id: Id, pos: Pos2) -> Pos2 {
        let animation_time = ui.style().animation_time;
        let x = ui.ctx().animate_value_with_time(id, pos.x, animation_time);
        pos2(x, pos.y)
    }

    pub(super) fn draw_button<T>(ui: &mut Ui, button: &ButtonSpace<T>, selected: bool) {
        let visuals = ui.style().interact_selectable(&button.response, selected);
        let animation_factor = animate_click(ui, &button.response);
        let radius = animation_factor * button.radius;
        let icon_radius = 0.5 * radius * animation_factor;
        let bg_fill = button_fill(&button.response, &visuals);

        let painter = ui.painter();
        painter.circle(button.center, radius, bg_fill, visuals.bg_stroke);
        (button.option.icon)(painter, button.center, icon_radius, visuals.fg_stroke.color);
    }

    // We want to avoid drawing a background when the button is either active itself or was previously active.
    fn button_fill(response: &Response, visuals: &WidgetVisuals) -> Color32 {
        if interacted(&response) {
            visuals.bg_fill
        } else {
            Color32::TRANSPARENT
        }
    }

    fn interacted(response: &Response) -> bool {
        response.clicked() || response.hovered() || response.has_focus()
    }

    fn animate_click(ui: &Ui, response: &Response) -> f32 {
        let ctx = ui.ctx();
        let animation_time = ui.style().animation_time;
        let value = if response.is_pointer_button_down_on() {
            0.9
        } else {
            1.0
        };
        ctx.animate_value_with_time(response.id, value, animation_time)
    }
}

mod accessibility {
    use super::*;
    use crate::accesskit::{NodeBuilder, NodeId as AccessKitId, Role};
    use crate::{Id, WidgetInfo, WidgetType};

    pub(super) fn attach_widget_info<T: PartialEq>(
        ui: &mut Ui,
        space: AllocatedSpace<T>,
        label: &str,
        value: &T,
    ) -> Response {
        let button_group = button_group(&space);

        configure_accesskit_radio_group(ui, space.rect, space.response.id, label);

        for button in space.buttons {
            let selected = value == &button.option.value;
            attach_widget_info_to_button(ui, button, &button_group, selected);
        }

        space.response
    }

    fn configure_accesskit_radio_group(ui: &mut Ui, rect: Rect, id: Id, label: &str) {
        ui.ctx().accesskit_node_builder(id, |builder| {
            builder.set_role(Role::RadioGroup);
            builder.set_bounds(to_accesskit_rect(rect));
            builder.set_name(label);
        });
    }

    fn button_group<T>(space: &AllocatedSpace<T>) -> Vec<AccessKitId> {
        space
            .buttons
            .iter()
            .map(|b| b.response.id.accesskit_id())
            .collect()
    }

    fn attach_widget_info_to_button<T>(
        ui: &mut Ui,
        button: ButtonSpace<T>,
        buttons: &[AccessKitId],
        selected: bool,
    ) {
        let response = button.response;
        let label = button.option.label;
        response.widget_info(|| button_widget_info(ui, label, selected));
        configure_accesskit_radio_button(ui, response.id, buttons);
        response.on_hover_text(label);
    }

    fn button_widget_info(ui: &Ui, label: &str, selected: bool) -> WidgetInfo {
        WidgetInfo::selected(WidgetType::RadioButton, ui.is_enabled(), selected, label)
    }

    fn configure_accesskit_radio_button(ui: &mut Ui, id: Id, group: &[AccessKitId]) {
        let writer = |b: &mut NodeBuilder| b.set_radio_group(group);
        ui.ctx().accesskit_node_builder(id, writer);
    }

    fn to_accesskit_rect(rect: Rect) -> accesskit::Rect {
        accesskit::Rect {
            x0: rect.min.x.into(),
            y0: rect.min.y.into(),
            x1: rect.max.x.into(),
            y1: rect.max.y.into(),
        }
    }
}