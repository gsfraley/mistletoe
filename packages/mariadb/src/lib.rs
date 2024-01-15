mod base;
mod deployment;
mod secret;
mod service;

use crate::base::*;
use crate::deployment::*;
use crate::secret::*;
use crate::service::*;

use indexmap::IndexMap;
use k8s_openapi::api::core::v1::ResourceRequirements;
use mistletoe_api::v1alpha1::{MistOutput, MistResult};
use mistletoe_bind::mistletoe_package;
use serde::Deserialize;

mistletoe_package! {"
  name: mariadb
  labels:
    mistletoe.dev/group: mistletoe
"}

fn default_image() -> String { "mariadb:latest".to_string() }
fn default_service_type() -> String { "ClusterIP".to_string() }

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Inputs {
    name: String,
    #[serde(default)]
    namespace: Option<String>,

    #[serde(default)]
    labels: IndexMap<String, String>,
    #[serde(default = "default_image")]
    image: String,
    #[serde(default)]
    resources: Option<ResourceRequirements>,
    #[serde(default = "default_service_type")]
    service_type: String,

    #[serde(default)]
    users: IndexMap<String, UserValue>,
}

#[derive(Deserialize)]
pub struct UserValue {
    #[serde(flatten, default = "UserAuthValue::Random")]
    auth: UserAuthValue,
}

#[derive(Deserialize)]
#[serde(tag = "authType")]
pub enum UserAuthValue {
    Hash {
        hash: String,
    },
    Password {
        password: String,
    },
    Random,
}

pub fn generate(inputs: Inputs) -> MistResult {
    let output = MistOutput::new()
        .with_files_from_map(generate_secret(&inputs)?.get_files())
        .with_files_from_map(generate_deployment(&inputs)?.get_files())
        .with_files_from_map(generate_service(&inputs)?.get_files());

    Ok(output)
}
