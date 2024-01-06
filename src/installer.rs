use anyhow::anyhow;
use kube::{Api, Client, Discovery, ResourceExt};
use kube::api::{PatchParams, Patch};
use kube::core::{DynamicObject, GroupVersionKind};
use kube::discovery::Scope;

pub const INSTALL_LABEL_KEY: &'static str = "mistletoe.dev/tied-to-install";

pub fn set_install_label(install_name: &str, input_yaml: &str) -> anyhow::Result<String> {
    let mut documents = Vec::new();

    let process_resource = |resource: &mut serde_yaml::Mapping| {
        let metadata = resource.get_mut(serde_yaml::Value::String("metadata".to_string()))
            .unwrap().as_mapping_mut().unwrap();

        let labels = if let Some(labels) = metadata.get_mut(serde_yaml::Value::String("labels".to_string())) {
            labels.as_mapping_mut().unwrap()
        } else {
            metadata.insert(
                serde_yaml::Value::String("labels".to_string()),
                serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));
            metadata.get_mut(serde_yaml::Value::String("labels".to_string())).unwrap().as_mapping_mut().unwrap()
        };

        labels.insert(
            serde_yaml::Value::String(INSTALL_LABEL_KEY.to_string()),
            serde_yaml::Value::String(install_name.to_string()));
        
        serde_yaml::to_string(&resource)
    };

    let resources = serde_yaml::from_str::<serde_yaml::Value>(input_yaml)?;
    match resources {
        serde_yaml::Value::Mapping(mut resource) => documents.push(process_resource(&mut resource)?),
        serde_yaml::Value::Sequence(resource_seq) => {
            for resource_value in resource_seq {
                if let serde_yaml::Value::Mapping(mut resource) = resource_value {
                    documents.push(process_resource(&mut resource)?);
                } else {
                    return Err(anyhow!("misformatted YAML, expected Mapping at base of document"));
                }
            }
        },
        _ => return Err(anyhow!("misformatted YAML, expected documents of YAML Mappings")),
    }

    Ok(documents.join("\n---\n"))
}

pub async fn install(processed_yaml: &str) -> anyhow::Result<()> {
    let client = Client::try_default().await?;
    let discover = Discovery::new(client.clone()).run().await?;
    let patch_params = PatchParams::apply("mistctl").force();

    let mut mappings = Vec::new();
    match serde_yaml::from_str::<serde_yaml::Value>(processed_yaml)? {
        serde_yaml::Value::Mapping(resource) => mappings.push(resource),
        serde_yaml::Value::Sequence(resource_seq) => {
            for resource_value in resource_seq {
                if let serde_yaml::Value::Mapping(resource) = resource_value {
                    mappings.push(resource);
                } else {
                    return Err(anyhow!("misformatted YAML, expected Mapping at base of document"))
                }
            }
        },
        _ => return Err(anyhow!("misformatted YAML, expected documents of YAML Mappings")),
    }

    let objs_results = mappings.into_iter()
        .map(serde_yaml::Value::Mapping)
        .map(serde_yaml::from_value::<DynamicObject>);

    // TODO: There's probably a cleaner way to unwrap Results without breaking the vec into a for-loop
    let mut objs = Vec::new();
    for obj_result in objs_results {
        objs.push(obj_result?);
    }

    for obj in objs {
        let gvk = if let Some(tm) = &obj.types {
            GroupVersionKind::try_from(tm)?
        } else {
            return Err(anyhow!("could not determine TypeMeta for {:?}", serde_yaml::to_string(&obj)))
        };

        if let Some((ar, ac)) = discover.resolve_gvk(&gvk) {
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
