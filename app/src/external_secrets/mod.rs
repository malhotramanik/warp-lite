// Most of this module is dead code on web as it is not possible to retrieve
// external secrets from the browser.
#![cfg_attr(target_family = "wasm", allow(dead_code, unused_variables))]


pub use cloud_object_models::ExternalSecret;
use itertools::Itertools;
use lazy_static::lazy_static;

use crate::ui_components::icons::Icon;

lazy_static! {
    // Used as a delimeter to separate metadata (such as names and references)
    // in cases the cli tool doesn't display secrets in a common format (i.e. json)
    static ref WARP_SECRET_DELIMITER: &'static str = "/warp-secret-delimeter/";
    static ref LASTPASS_LIST_SECRETS_COMMAND: Vec<String> = {
        vec![
            "lpass".to_owned(),
            "ls".to_owned(),
            format!("--format=%an{}%ai", *WARP_SECRET_DELIMITER),
        ]
    };
}








pub trait ExternalSecretManager {
    fn icon(&self) -> Icon;
}

impl ExternalSecretManager for ExternalSecret {
    fn icon(&self) -> Icon {
        match self {
            ExternalSecret::OnePassword(_) => Icon::OnePassword,
            ExternalSecret::LastPass(_) => Icon::LastPass,
        }
    }
}



