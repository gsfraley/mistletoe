use super::*;

use k8s_openapi::api::core::v1::{Service, ServiceSpec, ServicePort};

pub fn generate_service(inputs: &Inputs) -> MistResult {
    let mut metadata = generate_base_metadata(inputs);
    metadata.labels.as_mut().unwrap()
        .insert("app.kubernetes.io/component".to_string(), "service".to_string());
    
    let mut deployment_labels = generate_base_labels(inputs);
    deployment_labels.insert("app.kubernetes.io/component".to_string(), "deployment".to_string());

    let service = Service {
        metadata: metadata.clone(),

        spec: Some(ServiceSpec {
            selector: Some(deployment_labels.clone().into_iter().collect()),
            type_: Some(inputs.service_type.clone()),
            
            ports: Some(vec![
                ServicePort {
                    name: Some("default".to_string()),
                    port: 3306,
                    protocol: Some("TCP".to_string()),
                    ..Default::default()
                }
            ]),

            ..Default::default()
        }),

        ..Default::default()
    };

    let output = MistOutput::new()
        .with_file("service.yaml".to_string(), serde_yaml::to_string(&service)?);

    Ok(output)
}
