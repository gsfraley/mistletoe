use anyhow::anyhow;

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
            serde_yaml::Value::String("mistletoe.dev/installation".to_string()),
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
                    return Err(anyhow!("misformatted YAML, expected documents of YAML resources"));
                }
            }
        },
        _ => return Err(anyhow!("misformatted YAML, expected documents of YAML resources")),
    }

    Ok(documents.join("\n---\n"))
}
