use warp_core::ui::appearance::Appearance;
use warpui::elements::{
    Border, ConstrainedBox, Container, CornerRadius, Flex, MouseStateHandle, ParentElement, Radius,
    Shrinkable,
};
use warpui::ui_components::button::ButtonVariant;
use warpui::ui_components::components::{UiComponent, UiComponentStyles};
use warpui::{
    AppContext, Element, Entity, FocusContext, SingletonEntity, TypedActionView, View, ViewContext,
    ViewHandle,
};

use super::EnvVarSecretCommand;
use crate::editor::EditorView;

const COMMAND_EDITOR_MIN_LINES: f32 = 6.;
const SPAN_FONT_SIZE: f32 = 16.;
const BUTTON_FONT_SIZE: f32 = 14.;
const CORE_WIDTH: f32 = 400.;
const CORE_HEIGHT: f32 = 250.;
const CONTAINER_PADDING: f32 = 25.;
const ELEMENT_SPACING: f32 = 10.;
const EDITOR_DIVIDE: f32 = 6.;

const SECRET_SPAN: &str = "Secret command";
const SAVE_BUTTON_LABEL: &str = "Save";
const CANCEL_BUTTON_LABEL: &str = "Cancel";

#[derive(Debug, Clone)]
pub enum EnvVarCommandDialogAction {
    Close,
    SaveCommand,
}

#[derive(Debug, Clone)]
pub enum EnvVarCommandDialogEvent {
    Close,
    SaveCommand(EnvVarSecretCommand),
}

#[derive(Default)]
struct MouseStateHandles {
    cancel_button_mouse_state_handle: MouseStateHandle,
    save_button_mouse_state_handle: MouseStateHandle,
}

pub struct EnvVarCommandDialog {
    mouse_state_handles: MouseStateHandles,
    name_editor: ViewHandle<EditorView>,
    command_editor: ViewHandle<EditorView>,
}

impl EnvVarCommandDialog {




    fn save_command_and_close(&mut self, ctx: &mut ViewContext<Self>) {
        ctx.emit(EnvVarCommandDialogEvent::SaveCommand(EnvVarSecretCommand {
            name: self.name_editor.as_ref(ctx).buffer_text(ctx),
            command: self.command_editor.as_ref(ctx).buffer_text(ctx),
        }));
        self.close(ctx);
    }

    fn close(&mut self, ctx: &mut ViewContext<Self>) {
        self.name_editor
            .update(ctx, |buffer, ctx| buffer.clear_buffer(ctx));

        self.command_editor
            .update(ctx, |buffer, ctx| buffer.clear_buffer(ctx));
        ctx.emit(EnvVarCommandDialogEvent::Close)
    }

    fn should_disable_save(&self, app: &AppContext) -> bool {
        self.command_editor.as_ref(app).is_empty(app)
    }

    fn render_button(
        &self,
        appearance: &Appearance,
        button_mouse_state: MouseStateHandle,
        action: EnvVarCommandDialogAction,
        label_text: &str,
        is_save: bool,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let mut button = appearance
            .ui_builder()
            .button(
                if is_save {
                    ButtonVariant::Accent
                } else {
                    ButtonVariant::Secondary
                },
                button_mouse_state,
            )
            .with_centered_text_label(label_text.to_owned())
            .with_style(UiComponentStyles {
                font_size: Some(BUTTON_FONT_SIZE),
                font_weight: Some(warpui::fonts::Weight::Normal),
                ..Default::default()
            });

        if is_save && self.should_disable_save(app) {
            button = button.disabled();
        };

        button
            .build()
            .on_click(move |ctx, _, _| ctx.dispatch_typed_action(action.clone()))
            .with_cursor(warpui::platform::Cursor::PointingHand)
            .finish()
    }

    fn render_command_editor(&self, appearance: &Appearance, app: &AppContext) -> Box<dyn Element> {
        let line_height = self
            .command_editor
            .as_ref(app)
            .line_height(app.font_cache(), appearance);

        Container::new(
            ConstrainedBox::new(
                appearance
                    .ui_builder()
                    .text_input(self.command_editor.clone())
                    .with_style(UiComponentStyles {
                        ..Default::default()
                    })
                    .build()
                    .finish(),
            )
            .with_height(COMMAND_EDITOR_MIN_LINES * line_height)
            .finish(),
        )
        .with_margin_bottom(ELEMENT_SPACING)
        .finish()
    }

    fn render_name_editor(&self, appearance: &Appearance) -> Box<dyn Element> {
        Container::new(
            appearance
                .ui_builder()
                .text_input(self.name_editor.clone())
                .with_style(UiComponentStyles::default())
                .build()
                .finish(),
        )
        .with_margin_bottom(EDITOR_DIVIDE)
        .finish()
    }

    fn render_dialog_span(&self, appearance: &Appearance) -> Box<dyn Element> {
        Container::new(
            appearance
                .ui_builder()
                .span(SECRET_SPAN)
                .with_style(UiComponentStyles {
                    font_size: Some(SPAN_FONT_SIZE),
                    ..Default::default()
                })
                .build()
                .finish(),
        )
        .with_margin_bottom(ELEMENT_SPACING)
        .finish()
    }
}

impl Entity for EnvVarCommandDialog {
    type Event = EnvVarCommandDialogEvent;
}

impl View for EnvVarCommandDialog {
    fn ui_name() -> &'static str {
        "EnvVarCommandDialog"
    }

    fn on_focus(&mut self, focus_ctx: &FocusContext, ctx: &mut ViewContext<Self>) {
        if focus_ctx.is_self_focused() {
            ctx.focus(&self.name_editor);
            ctx.notify();
        }
    }

    fn render(&self, app: &AppContext) -> Box<dyn Element> {
        let appearance = Appearance::as_ref(app);

        ConstrainedBox::new(
            Shrinkable::new(
                1.,
                Container::new(
                    Flex::column()
                        .with_child(self.render_dialog_span(appearance))
                        .with_child(self.render_name_editor(appearance))
                        .with_child(self.render_command_editor(appearance, app))
                        .with_child(
                            Flex::row()
                                .with_child(
                                    Shrinkable::new(
                                        1.,
                                        Container::new(
                                            self.render_button(
                                                appearance,
                                                self.mouse_state_handles
                                                    .cancel_button_mouse_state_handle
                                                    .clone(),
                                                EnvVarCommandDialogAction::Close,
                                                CANCEL_BUTTON_LABEL,
                                                false,
                                                app,
                                            ),
                                        )
                                        .with_margin_right(ELEMENT_SPACING)
                                        .finish(),
                                    )
                                    .finish(),
                                )
                                .with_child(
                                    Shrinkable::new(
                                        1.,
                                        self.render_button(
                                            appearance,
                                            self.mouse_state_handles
                                                .save_button_mouse_state_handle
                                                .clone(),
                                            EnvVarCommandDialogAction::SaveCommand,
                                            SAVE_BUTTON_LABEL,
                                            true,
                                            app,
                                        ),
                                    )
                                    .finish(),
                                )
                                .finish(),
                        )
                        .finish(),
                )
                .with_corner_radius(CornerRadius::with_all(Radius::Pixels(8.)))
                .with_uniform_padding(CONTAINER_PADDING)
                .with_border(Border::all(2.).with_border_fill(appearance.theme().surface_2()))
                .with_background(appearance.theme().surface_1())
                .finish(),
            )
            .finish(),
        )
        .with_max_width(CORE_WIDTH)
        .with_height(CORE_HEIGHT)
        .finish()
    }
}

impl TypedActionView for EnvVarCommandDialog {
    type Action = EnvVarCommandDialogAction;

    fn handle_action(&mut self, action: &Self::Action, ctx: &mut ViewContext<Self>) {
        match action {
            EnvVarCommandDialogAction::Close => self.close(ctx),
            EnvVarCommandDialogAction::SaveCommand => self.save_command_and_close(ctx),
        }
    }
}
