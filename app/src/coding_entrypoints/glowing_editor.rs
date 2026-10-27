use pathfinder_color::ColorU;
use pathfinder_geometry::vector::Vector2F;
use warp_core::ui::Icon;
use warp_core::ui::appearance::Appearance;
use warpui::elements::{
    Align, ConstrainedBox, Container, CornerRadius, CrossAxisAlignment, DropShadow, Flex,
    MainAxisAlignment, ParentElement as _, Radius, Shrinkable,
};
use warpui::fonts::Weight;
use warpui::ui_components::components::{BorderStyle, Coords, UiComponent as _, UiComponentStyles};
use warpui::{
    AppContext, Element, Entity, FocusContext, SingletonEntity as _, TypedActionView, View,
    ViewContext, ViewHandle,
};

use crate::editor::EditorView;

const PROMPT_INPUT_HEIGHT: f32 = 56.;
const ICON_MARGIN_LEFT: f32 = 12.;
const ICON_MARGIN_RIGHT: f32 = 6.;

pub struct GlowingEditor {
    editor: ViewHandle<EditorView>,
    /// A closure that returns if the current editor content are valid and can be submitted.
    validator: Box<dyn Fn(&str) -> bool>,
    /// Whether or not the last submission attempt failed validation.
    has_error: bool,
}

impl GlowingEditor {

    /// Validates the input contents using the provided `validator` whenever a submit action is
    /// attempted.
    #[expect(dead_code, reason = "Nothing needs validation currently.")]
    pub fn with_validator<F: Fn(&str) -> bool + 'static>(mut self, validator: F) -> Self {
        self.validator = Box::new(validator);
        self
    }


}

pub enum GlowingEditorEvent {
}

impl Entity for GlowingEditor {
    type Event = GlowingEditorEvent;
}

impl TypedActionView for GlowingEditor {
    type Action = ();
}

impl View for GlowingEditor {
    fn ui_name() -> &'static str {
        "GlowingEditor"
    }

    fn on_focus(&mut self, focus_ctx: &FocusContext, ctx: &mut ViewContext<Self>) {
        if focus_ctx.is_self_focused() {
            ctx.focus(&self.editor);
        }
    }

    fn render(&self, app: &AppContext) -> Box<dyn Element> {
        let appearance = Appearance::as_ref(app);
        let theme = appearance.theme();

        let input_box = Shrinkable::new(
            1.,
            Align::new(
                appearance
                    .ui_builder()
                    .text_input(self.editor.clone())
                    .with_style(UiComponentStyles {
                        font_weight: Some(Weight::Semibold),
                        border_style: Some(BorderStyle::None),
                        border_width: Some(0.),
                        background: Some(ColorU::transparent_black().into()),
                        padding: Some(Coords::uniform(10.).left(0.)),
                        ..Default::default()
                    })
                    .build()
                    .finish(),
            )
            .finish(),
        )
        .finish();

        let font_size = appearance.ui_font_size() + 2.;
        let agent_icon = Container::new(
            ConstrainedBox::new(
                Icon::AgentMode
                    .to_warpui_icon(theme.sub_text_color(theme.background()))
                    .finish(),
            )
            .with_height(font_size)
            .with_width(font_size)
            .finish(),
        )
        .with_margin_left(ICON_MARGIN_LEFT)
        .with_margin_right(ICON_MARGIN_RIGHT)
        .finish();

        let editor_content = ConstrainedBox::new(
            Flex::row()
                .with_cross_axis_alignment(CrossAxisAlignment::Center)
                .with_main_axis_alignment(MainAxisAlignment::Start)
                .with_children([agent_icon, input_box])
                .finish(),
        )
        .with_min_height(PROMPT_INPUT_HEIGHT);

        let border_fill = if self.has_error {
            theme.ui_error_color()
        } else {
            theme.outline().into_solid()
        };

        let shadow_color = if self.has_error {
            ColorU::new(255, 0, 0, 100) // Red shadow with higher opacity for error
        } else {
            ColorU::new(255, 143, 253, 15) // Default purple shadow
        };

        Container::new(editor_content.finish())
            .with_border(border_fill)
            .with_corner_radius(CornerRadius::with_all(Radius::Pixels(4.)))
            .with_drop_shadow(
                DropShadow::new_with_standard_offset_and_spread(shadow_color)
                    .with_offset(Vector2F::zero()),
            )
            .with_background(theme.background().into_solid())
            .finish()
    }
}
