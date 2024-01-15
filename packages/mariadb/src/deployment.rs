use super::*;

use k8s_openapi::api::apps::v1::{Deployment, DeploymentSpec};
use k8s_openapi::api::core::v1::{
    PodTemplateSpec, PodSpec,
    Container, ContainerPort,
    EnvFromSource, SecretEnvSource
};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{LabelSelector, ObjectMeta};

pub fn generate_deployment(inputs: &Inputs) -> MistResult {
    let mut metadata = generate_base_metadata(inputs);
    metadata.labels.as_mut().unwrap()
        .insert("app.kubernetes.io/component".to_string(), "deployment".to_string());

    let deployment = Deployment {
        metadata: metadata.clone(),

        spec: Some(DeploymentSpec {
            replicas: Some(1),
            strategy: None,

            selector: LabelSelector {
                match_labels: metadata.labels.clone(),
                ..Default::default()
            },

            template: PodTemplateSpec {
                metadata: Some(ObjectMeta {
                    labels: Some(metadata.labels.clone().unwrap()),
                    ..Default::default()
                }),

                spec: Some(PodSpec {
                    containers: vec![
                        Container {
                            name: "mariadb".to_string(),
                            image: Some(inputs.image.clone()),
                            resources: inputs.resources.clone(),

                            ports: Some(vec![
                                ContainerPort {
                                    name: Some("mariadb".to_string()),
                                    container_port: 3306,
                                    protocol: Some("TCP".to_string()),
                                    ..Default::default()
                                }
                            ]),

                            env_from: Some(vec![
                                EnvFromSource {
                                    secret_ref: Some(SecretEnvSource {
                                        name: Some(format!("{}-secret-env", inputs.name.clone())),
                                        ..Default::default()
                                    }),

                                    ..Default::default()
                                }
                            ]),

                            ..Default::default()
                        }
                    ],

                    ..Default::default()
                }),
            },

            ..Default::default()
        }),

        ..Default::default()
    };
        

    let output = MistOutput::new()
        .with_file("deployment.yaml".to_string(), serde_yaml::to_string(&deployment)?);

    Ok(output)
}

