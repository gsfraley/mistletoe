use super::*;

use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;

pub fn generate_base_metadata(inputs: &Inputs) -> ObjectMeta {
    ObjectMeta {
        name: Some(inputs.name.clone()),
        namespace: inputs.namespace.clone(),
        labels: Some(generate_base_labels(inputs).into_iter().collect()),
        ..Default::default()
    }
}

pub fn generate_base_labels(inputs: &Inputs) -> IndexMap<String, String> {
    let mut labels = IndexMap::new();
    labels.insert("app".to_string(), inputs.name.clone());
    labels.insert("app.kubernetes.io/name".to_string(), inputs.name.clone());

    inputs.labels.clone().into_iter().for_each(|(k, v)| {
        labels.insert(k, v);
    });

    labels
}
