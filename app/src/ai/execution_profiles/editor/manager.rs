use std::collections::HashMap;

use warpui::{Entity, SingletonEntity, WindowId};

use crate::PaneViewLocator;
use crate::ai::execution_profiles::ExecutionProfileId;

/// Manages execution profile editor panes across different windows and profiles.
///
/// This manager tracks which execution profile editor panes are active in each window,
/// allowing the application to locate and interact with these panes when needed.
/// It maintains a mapping from each window ID to a map from profile IDs to pane data,
/// including the locator information needed to find and reference specific panes within their pane groups.
#[derive(Default)]
pub struct ExecutionProfileEditorManager {
    panes: HashMap<WindowId, HashMap<ExecutionProfileId, ExecutionProfileEditorPaneData>>,
}

#[derive(Clone, Copy)]
struct ExecutionProfileEditorPaneData {
    locator: PaneViewLocator,
}

impl ExecutionProfileEditorManager {


}

impl Entity for ExecutionProfileEditorManager {
    type Event = ();
}

impl SingletonEntity for ExecutionProfileEditorManager {}
