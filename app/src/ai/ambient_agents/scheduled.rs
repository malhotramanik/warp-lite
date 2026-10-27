use std::collections::HashMap;

pub use cloud_object_models::{
    CloudScheduledAmbientAgent, CloudScheduledAmbientAgentModel, ScheduledAmbientAgent,
};
use futures::channel::oneshot;
use warpui::{Entity, ModelContext, ModelHandle, SingletonEntity};

use crate::cloud_object::model::generic_string_model::StringModel;
use crate::cloud_object::model::json_model::JsonModel;
use crate::cloud_object::{
    GenericStringObjectFormat, GenericStringObjectUniqueKey,
    JsonObjectType, Revision,
};
use crate::server::cloud_objects::update_manager::{
    ObjectOperation, OperationSuccessType, UpdateManager, UpdateManagerEvent,
};
use crate::server::ids::SyncId;
use crate::server::sync_queue::QueueItem;

impl StringModel for ScheduledAmbientAgent {
    type CloudObjectType = CloudScheduledAmbientAgent;

    fn model_type_name(&self) -> &'static str {
        "Scheduled ambient agent"
    }

    fn should_enforce_revisions() -> bool {
        true
    }

    fn model_format() -> GenericStringObjectFormat {
        GenericStringObjectFormat::Json(JsonObjectType::ScheduledAmbientAgent)
    }

    fn display_name(&self) -> String {
        self.name.clone()
    }

    fn update_object_queue_item(
        &self,
        revision_ts: Option<Revision>,
        object: &CloudScheduledAmbientAgent,
    ) -> QueueItem {
        QueueItem::UpdateScheduledAmbientAgent {
            model: object.model().clone().into(),
            id: object.id,
            revision: revision_ts.or(object.metadata.revision),
        }
    }

    fn uniqueness_key(&self) -> Option<GenericStringObjectUniqueKey> {
        None
    }

    fn should_show_activity_toasts() -> bool {
        false
    }

    fn warn_if_unsaved_at_quit() -> bool {
        true
    }
}

impl JsonModel for ScheduledAmbientAgent {
    fn json_object_type() -> JsonObjectType {
        JsonObjectType::ScheduledAmbientAgent
    }
}


pub struct ScheduledAgentManager {
    /// Mapping of channels for schedules that are pending deletion, so that we can
    /// report when they complete from the CLI.
    pending_deletes: HashMap<SyncId, oneshot::Sender<anyhow::Result<()>>>,
}

#[cfg_attr(target_family = "wasm", allow(dead_code))]
impl ScheduledAgentManager {
    pub fn new(ctx: &mut ModelContext<Self>) -> Self {
        ctx.subscribe_to_model(
            &UpdateManager::handle(ctx),
            Self::handle_update_manager_event,
        );

        Self {
            pending_deletes: Default::default(),
        }
    }



    fn handle_update_manager_event(
        &mut self,
        _: ModelHandle<UpdateManager>,
        event: &UpdateManagerEvent,
        _ctx: &mut ModelContext<Self>,
    ) {
        if let UpdateManagerEvent::ObjectOperationComplete { result } = event
            && let ObjectOperation::Delete { .. } = result.operation
            && let Some(server_id) = result.server_id
        {
            let sync_id = SyncId::ServerId(server_id);
            if let Some(tx) = self.pending_deletes.remove(&sync_id) {
                match result.success_type {
                    OperationSuccessType::Success => {
                        let _ = tx.send(Ok(()));
                    }
                    OperationSuccessType::Failure => {
                        let _ = tx.send(Err(anyhow::anyhow!(
                            "Failed to delete scheduled ambient agent"
                        )));
                    }
                    OperationSuccessType::Denied(ref message) => {
                        let _ = tx.send(Err(anyhow::anyhow!("Deletion denied: {}", message)));
                    }
                    OperationSuccessType::Rejection => {
                        let _ = tx.send(Err(anyhow::anyhow!("Deletion rejected by server")));
                    }
                    OperationSuccessType::FeatureNotAvailable => {
                        let _ = tx.send(Err(anyhow::anyhow!(
                            "Scheduled ambient agents not available"
                        )));
                    }
                }
            }
        }
    }






}

impl Entity for ScheduledAgentManager {
    type Event = ();
}

impl SingletonEntity for ScheduledAgentManager {}
