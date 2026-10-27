use std::collections::HashMap;

use warpui::{Entity, ModelContext, ModelHandle, SingletonEntity, WeakViewHandle};

use crate::cloud_object::model::persistence::CloudModel;
use crate::env_vars::view::env_var_collection::EnvVarCollectionView;
use crate::pane_group::{ PaneContent};
use crate::server::cloud_objects::update_manager::{
    ObjectOperation, OperationSuccessType, UpdateManager, UpdateManagerEvent,
};
use crate::server::ids::SyncId;
use crate::{PaneViewLocator, WindowId};

pub struct EnvVarCollectionManager {
    panes_by_hashed_id: HashMap<String, EnvVarCollectionPaneData>,
}


/// Manages EnvVarCollection panes
impl EnvVarCollectionManager {
    pub fn new(ctx: &mut ModelContext<Self>) -> Self {
        ctx.subscribe_to_model(
            &UpdateManager::handle(ctx),
            Self::handle_update_manager_event,
        );

        EnvVarCollectionManager {
            panes_by_hashed_id: HashMap::new(),
        }
    }






    fn handle_update_manager_event(
        &mut self,
        _: ModelHandle<UpdateManager>,
        event: &UpdateManagerEvent,
        ctx: &mut ModelContext<Self>,
    ) {
        let UpdateManagerEvent::ObjectOperationComplete { result } = event else {
            return;
        };

        if !matches!(&result.success_type, OperationSuccessType::Success) {
            return;
        }
        if let ObjectOperation::Create { .. } = result.operation {
            let server_id = result.server_id.expect("Expect server id on success");
            let Some(server_id) = CloudModel::as_ref(ctx)
                .get_env_var_collection_by_uid(&server_id.uid())
                .and_then(|collection| collection.id.into_server())
            else {
                return;
            };
            let Some(client_id) = result.client_id else {
                return;
            };

            if let Some(mut pane) = self.panes_by_hashed_id.remove(&client_id.to_string()) {
                pane.env_var_collection_id = SyncId::ServerId(server_id);
                self.panes_by_hashed_id
                    .insert(server_id.uid().clone(), pane);
            }
        }
    }

    pub fn reset(&mut self) {
        self.panes_by_hashed_id.clear();
    }
}

struct EnvVarCollectionPaneData {
    env_var_collection_id: SyncId,
    window_id: WindowId,
    handle: WeakViewHandle<EnvVarCollectionView>,
    locator: PaneViewLocator,
}

impl Entity for EnvVarCollectionManager {
    type Event = ();
}

impl SingletonEntity for EnvVarCollectionManager {}
