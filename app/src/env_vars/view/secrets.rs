use pathfinder_geometry::vector::vec2f;
use warp_core::features::FeatureFlag;
use warp_core::ui::appearance::Appearance;
#[cfg(not(target_family = "wasm"))]
use warpui::SingletonEntity;
use warpui::elements::{
    ChildAnchor, ChildView, Clipped, ConstrainedBox, Container, Empty, Fill, MainAxisAlignment,
    MainAxisSize, MouseStateHandle, OffsetPositioning, ParentAnchor, ParentElement,
    ParentOffsetBounds, Shrinkable, Stack,
};
use warpui::fonts::Weight;
use warpui::ui_components::button::{ButtonVariant, TextAndIcon, TextAndIconAlignment};
use warpui::ui_components::components::{UiComponent, UiComponentStyles};
use warpui::{Element, ViewContext};

use super::env_var_collection::{
    CORE_MAX_WIDTH, EnvVarCollectionAction, EnvVarCollectionView, ROW_SPACING, VariableRowIndex,
};
use crate::drive::sharing::ContentEditability;
use crate::env_vars::EnvVarValue;
use crate::external_secrets::{ExternalSecretManager};
use crate::ui_components::icons::Icon;
#[cfg(all(not(target_family = "wasm"), feature = "local_tty"))]
use crate::{
    terminal::local_shell::LocalShellState,
    view_components::{DismissibleToast, ToastLink},
    workspace::{ToastStack, WorkspaceAction},
};

impl EnvVarCollectionView {



    pub(super) fn render_secret_or_command_button(
        &self,
        appearance: &Appearance,
        secret: &EnvVarValue,
        menu_button_mouse_state: MouseStateHandle,
        row_index: usize,
        is_focused: bool,
        editability: ContentEditability,
    ) -> Box<dyn Element> {
        let (display_name, action, menu, icon) = match secret {
            EnvVarValue::Secret(sec) => (
                sec.get_display_name(),
                EnvVarCollectionAction::DisplayRenderedSecretMenu(VariableRowIndex(row_index)),
                &self.menus.rendered_secret_menu,
                sec.icon(),
            ),
            EnvVarValue::Command(cmd) => (
                if !cmd.name.is_empty() {
                    cmd.name.clone()
                } else {
                    cmd.command.clone()
                },
                EnvVarCollectionAction::DisplayRenderedCommandMenu(VariableRowIndex(row_index)),
                &self.menus.rendered_command_menu,
                Icon::Terminal,
            ),
            _ => {
                log::warn!("Secret type not supported for button rendering");
                return Empty::new().finish();
            }
        };

        let text_and_icon = TextAndIcon::new(
            TextAndIconAlignment::IconFirst,
            display_name,
            icon.to_warpui_icon(appearance.theme().active_ui_text_color()),
            MainAxisSize::Max,
            MainAxisAlignment::Center,
            vec2f(16., 16.),
        )
        .with_inner_padding(4.);

        let default_button_styles = UiComponentStyles {
            font_size: Some(appearance.ui_font_size()),
            font_family_id: Some(appearance.ui_font_family()),
            font_color: Some(
                appearance
                    .theme()
                    .main_text_color(appearance.theme().background())
                    .into(),
            ),
            border_width: Some(1.),
            border_color: Some(appearance.theme().surface_2().into()),
            font_weight: Some(Weight::Bold),
            background: Some(Fill::None),
            ..Default::default()
        };

        let hovered_styles = UiComponentStyles {
            border_width: Some(1.),
            border_color: Some(appearance.theme().accent_button_color().into()),
            ..default_button_styles
        };

        let mut button = appearance
            .ui_builder()
            .button(ButtonVariant::Outlined, menu_button_mouse_state)
            .with_style(default_button_styles)
            .with_hovered_styles(hovered_styles)
            .with_text_and_icon_label(text_and_icon);

        if FeatureFlag::SharedWithMe.is_enabled() && !editability.can_edit() {
            button = button.disabled();
        }

        let button = button
            .build()
            .on_click(move |ctx, _, _| ctx.dispatch_typed_action(action.clone()));

        Shrinkable::new(
            1.,
            Container::new(
                ConstrainedBox::new({
                    let mut stack = Stack::new().with_child(Clipped::new(button.finish()).finish());
                    if is_focused {
                        stack.add_positioned_overlay_child(
                            ChildView::new(menu).finish(),
                            OffsetPositioning::offset_from_parent(
                                vec2f(0., 0.),
                                ParentOffsetBounds::WindowByPosition,
                                ParentAnchor::TopRight,
                                ChildAnchor::TopLeft,
                            ),
                        );
                    }
                    stack.finish()
                })
                .with_width(CORE_MAX_WIDTH)
                .finish(),
            )
            .with_margin_right(ROW_SPACING)
            .finish(),
        )
        .finish()
    }
}
