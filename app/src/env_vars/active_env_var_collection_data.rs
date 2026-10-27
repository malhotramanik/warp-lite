use warpui::{Entity, SingletonEntity};

use crate::cloud_object::model::view::CloudViewModel;
use crate::cloud_object::{CloudObject, Revision, Space};
use crate::drive::sharing::{ContentEditability, SharingAccessLevel};
use crate::server::ids::SyncId;
use crate::{AppContext, CloudModel};

#[derive(Default, Clone)]
pub enum ActiveEnvVarCollection {
    #[default]
    None,
    // An EnvVarCollection already stored in CloudModel, all relevant data should be queried
    // from CloudModel directly
    CommittedEnvVarCollection(SyncId),
    // An EnvVarCollection that has been created and displayed in the view, but is not yet
    // committed to CloudModel
}

#[derive(Default, PartialEq, Debug)]
pub enum SavingStatus {
    #[default]
    Saved,
    Unsaved,
}

#[derive(Default)]
pub struct ActiveEnvVarCollectionData {
    pub saving_status: SavingStatus,
    pub active_env_var_collection: ActiveEnvVarCollection,
    pub revision_ts: Option<Revision>,
}

impl ActiveEnvVarCollectionData {






    pub fn id(&self) -> Option<SyncId> {
        match &self.active_env_var_collection {
            ActiveEnvVarCollection::None => None,
            ActiveEnvVarCollection::CommittedEnvVarCollection(id) => Some(*id),
        }
    }

    /// The current user's access level on this env var collection.
    pub fn access_level(&self, app: &AppContext) -> SharingAccessLevel {
        match &self.active_env_var_collection {
            ActiveEnvVarCollection::CommittedEnvVarCollection(sync_id) => {
                CloudViewModel::as_ref(app).access_level(&sync_id.uid(), app)
            }
            ActiveEnvVarCollection::None  => {
                SharingAccessLevel::Full
            }
        }
    }

    pub fn editability(&self, app: &AppContext) -> ContentEditability {
        match &self.active_env_var_collection {
            ActiveEnvVarCollection::CommittedEnvVarCollection(sync_id) => {
                CloudViewModel::as_ref(app).object_editability(&sync_id.uid(), app)
            }
            ActiveEnvVarCollection::None  => {
                ContentEditability::Editable
            }
        }
    }

    /// The space that this env var collection is in.
    pub fn space(&self, app: &AppContext) -> Option<Space> {
        match &self.active_env_var_collection {
            ActiveEnvVarCollection::None => None,
            ActiveEnvVarCollection::CommittedEnvVarCollection(sync_id) => {
                CloudViewModel::as_ref(app).object_space(&sync_id.uid(), app)
            }
        }
    }

    pub fn active_env_var_collection(&self) -> ActiveEnvVarCollection {
        self.active_env_var_collection.clone()
    }

    /// Whether or not the EVC has been synced to the server.
    pub fn is_on_server(&self) -> bool {
        matches!(
            &self.active_env_var_collection,
            ActiveEnvVarCollection::CommittedEnvVarCollection(SyncId::ServerId(_))
        )
    }



    pub fn trash_status(&self, ctx: &AppContext) -> TrashStatus {
        match &self.active_env_var_collection {
            ActiveEnvVarCollection::None  => {
                TrashStatus::Active
            }
            ActiveEnvVarCollection::CommittedEnvVarCollection(id) => {
                let cloud_model = CloudModel::as_ref(ctx);
                match cloud_model.get_env_var_collection(id) {
                    Some(env_var_collection) => {
                        if env_var_collection.is_trashed(cloud_model) {
                            TrashStatus::Trashed
                        } else {
                            TrashStatus::Active
                        }
                    }
                    None => TrashStatus::Deleted,
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrashStatus {
    Active,
    Trashed,
    Deleted,
}

pub enum ActiveEnvVarCollectionDataEvent {
}

impl Entity for ActiveEnvVarCollectionData {
    type Event = ActiveEnvVarCollectionDataEvent;
}
