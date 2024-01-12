use std::collections::BTreeMap;

use anyhow::anyhow;
use k8s_openapi::api::core::v1::Namespace;
use kube::{Client, Discovery, Api, ResourceExt};
use kube::api::{ListParams, PatchParams, Patch, DeleteParams};
use kube::core::{DynamicObject, GroupVersionKind};
use kube::discovery::{ApiResource, ApiCapabilities, Scope};
use serde_yaml::Mapping;

pub const TIED_TO_INSTALL_NAME_KEY: &'static str = "mistletoe.dev/tied-to-install-name";
pub const TIED_TO_INSTALL_VERSION_KEY: &'static str = "mistletoe.dev/tied-to-install-version";

pub struct InstallRef {
    pub name: String,
    pub version: Option<u32>,
}

impl InstallRef {
    pub async fn get_resources(&self) -> anyhow::Result<Vec<DynamicObject>> {
        let client = Client::try_default().await?;
        let discovery = Discovery::new(client.clone()).run().await?;

        let namespaces = Api::<Namespace>::all(client.clone())
            .list(&ListParams::default()).await?;

        let resource_types: Vec<(ApiResource, ApiCapabilities)> = discovery.groups()
            .flat_map(|group| group.versions()
                .flat_map(|version| group.versioned_resources(version)))
            .collect();

        let mut label_selector = format!("{}={}", TIED_TO_INSTALL_NAME_KEY, self.name);
        if let Some(version) = self.version {
            label_selector += &format!(",{}=v{}", TIED_TO_INSTALL_VERSION_KEY, version);
        }

        let label_list_params = ListParams::default().labels(&label_selector);
        let mut resources = Vec::new();

        for (resource_type, resource_caps) in resource_types {
            if !resource_caps.supports_operation("list") { continue }

            if resource_caps.scope == Scope::Cluster {
                let api = Api::<DynamicObject>::all_with(
                    client.clone(),
                    &resource_type);

                // TODO: This and the equivalent block in the other case are awful -- we're actively ignoring
                // list-method results that don't conform, but this could potentially result in us not finding
                // resources.  We should properly analyze the API or break apart error cases to ensure robustness.
                if let Ok(object_list) = api.list(&label_list_params).await {
                    for item in object_list.items {
                        resources.push(api.get(&item.name_any()).await?);
                    }
                }
            } else {
                for namespace in &namespaces {
                    let api = Api::<DynamicObject>::namespaced_with(
                        client.clone(),
                        namespace.metadata.name.as_ref().unwrap(),
                        &resource_type);

                    if let Ok(object_list) = api.list(&label_list_params).await {
                        for item in object_list.items {
                            resources.push(api.get(&item.name_any()).await?);
                        }
                    }
                }
            }
        }

        Ok(resources)
    }

    pub async fn apply_resources(&self, install_resources: &InstallResources) -> anyhow::Result<()> {
        let client = Client::try_default().await?;
        let discovery = Discovery::new(client.clone()).run().await?;
        let patch_params = PatchParams::apply("mistctl").force();

        for obj in &install_resources.resources {
            let gvk = if let Some(tm) = &obj.types {
                GroupVersionKind::try_from(tm)?
            } else {
                return Err(anyhow!("could not determine TypeMeta for {:?}", serde_yaml::to_string(&obj)));
            };

            if let Some((ar, ac)) = discovery.resolve_gvk(&gvk) {
                let api: Api<DynamicObject> = if ac.scope == Scope::Cluster {
                    Api::all_with(client.clone(), &ar)
                } else if let Some(namespace) = &obj.metadata.namespace {
                    Api::namespaced_with(client.clone(), namespace, &ar)
                } else {
                    Api::default_namespaced_with(client.clone(), &ar)
                };

                api.patch(
                    &obj.name_any(),
                    &patch_params,
                    &Patch::Apply(serde_yaml::to_value(&obj)?))
                    .await?;
            }
        }

        Ok(())
    }

    pub async fn delete_resources(&self) -> anyhow::Result<()> {
        let resources = self.get_resources().await?;

        let client = Client::try_default().await?;
        let discovery = Discovery::new(client.clone()).run().await?;
        let delete_params = DeleteParams::foreground();

        for resource in resources {
            let gvk = if let Some(tm) = &resource.types {
                GroupVersionKind::try_from(tm)?
            } else {
                // TODO: This is a bit of a hack, but some returned resources don't have types, and
                // we can't do anything about them -- probably worth investigating how they get here.
                continue;
            };

            if let Some((ar, ac)) = discovery.resolve_gvk(&gvk) {
                let api: Api<DynamicObject> = if ac.scope == Scope::Cluster {
                    Api::all_with(client.clone(), &ar)
                } else if let Some(namespace) = &resource.metadata.namespace {
                    Api::namespaced_with(client.clone(), namespace, &ar)
                } else {
                    Api::default_namespaced_with(client.clone(), &ar)
                };

                api.delete(
                    &resource.name_any(),
                    &delete_params)
                    .await?;
            }
        }

        Ok(())
    }
}

pub struct InstallResources {
    pub resources: Vec<DynamicObject>,
}

impl InstallResources {
    pub fn from_str(resources_str: &str) -> anyhow::Result<Self> {
        let documents = match serde_yaml::from_str::<serde_yaml::Value>(resources_str)? {
            serde_yaml::Value::Mapping(document) => vec![document],
            serde_yaml::Value::Sequence(documents) => documents.into_iter()
                .map(|value| if let serde_yaml::Value::Mapping(document) = value {
                    Ok(document)
                } else {
                    Err(anyhow!("unexpected YAML document value {:?}", value))
                })
                .collect::<Result<Vec<Mapping>, _>>()?,
            value => return Err(anyhow!("unexpected root YAML value {:?}", value)),
        };

        Ok(Self { resources: documents.into_iter()
            .map(|document| serde_yaml::Value::Mapping(document))
            .map(|value| serde_yaml::from_value(value))
            .collect::<Result<Vec<DynamicObject>, _>>()? })
    }

    pub fn to_string(&self) -> anyhow::Result<String> {
        Ok(self.resources.iter()
            .map(serde_yaml::to_string)
            .collect::<Result<Vec<String>, _>>()?
            .join("\n---\n"))
    }

    pub fn label_resources(mut self, labels: BTreeMap<String, String>) -> anyhow::Result<Self> {
        for resource in &mut self.resources {
            let resource_labels = resource.metadata.labels.get_or_insert_with(|| BTreeMap::new());
            resource_labels.append(&mut labels.clone());
        }

        Ok(self)
    }

    pub fn label_resources_with(self, name: &str, version: u32) -> anyhow::Result<Self> {
        let mut labels = BTreeMap::new();
        labels.insert(TIED_TO_INSTALL_NAME_KEY.to_string(), name.to_string());
        labels.insert(TIED_TO_INSTALL_VERSION_KEY.to_string(), format!("v{}", version));

        self.label_resources(labels)
    }
}
