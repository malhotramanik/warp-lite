use itertools::Itertools;
use warpui::AppContext;

use super::searcher::ExternalSecretSearchItemAction;
use crate::external_secrets::ExternalSecret;
use crate::search::data_source::{Query, QueryResult};
use crate::search::mixer::{DataSourceRunErrorWrapper, SyncDataSource};



