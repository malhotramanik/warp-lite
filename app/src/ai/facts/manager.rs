use std::collections::HashMap;

use warpui::{Entity, SingletonEntity, ViewHandle, WindowId};

use crate::PaneViewLocator;
use crate::ai::facts::AIFactView;

/// Singleton model to manage state of AI fact panes across multiple windows
/// (where only one AI fact pane can exist per window). Specifically:
/// - Maintains AI fact view handles to preserve state when panes are hidden
/// - Tracks currently open AI fact panes and their location
#[derive(Default)]
pub struct AIFactManager {
    panes: HashMap<WindowId, AIFactPaneData>,
}

struct AIFactPaneData {
    locator: Option<PaneViewLocator>,
    view: ViewHandle<AIFactView>,
}

impl AIFactManager {
    pub fn new() -> Self {
        Self::default()
    }


    pub fn register_view(&mut self, window_id: WindowId, view: ViewHandle<AIFactView>) {
        if let Some(data) = self.panes.get_mut(&window_id) {
            data.view = view;
        } else {
            self.panes.insert(
                window_id,
                AIFactPaneData {
                    locator: None,
                    view,
                },
            );
        }
    }



}

impl Entity for AIFactManager {
    type Event = ();
}

impl SingletonEntity for AIFactManager {}
