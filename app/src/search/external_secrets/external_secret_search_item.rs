use ordered_float::OrderedFloat;
use warpui::elements::{ConstrainedBox, Container, Highlight, Text};
use warpui::fonts::{Properties, Weight};
use warpui::{AppContext, Element, SingletonEntity};

use super::searcher::ExternalSecretSearchItemAction;
use crate::appearance::Appearance;
use crate::external_secrets::{ExternalSecret, ExternalSecretManager};
use crate::search::external_secrets::view::styles;
use crate::search::item::{IconLocation, SearchItem};
use crate::search::result_renderer::ItemHighlightState;



