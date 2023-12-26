use mistletoe_api::v0_1::{MistResult, MistResultFiles};
use mistletoe_bind::misthusk_headers;

use indoc::formatdoc;
use serde::Deserialize;

misthusk_headers! {"
  name: example-nginx
  labels:
    mistletoe.dev/group: mistletoe-examples
"}

#[derive(Deserialize)]
pub struct InputConfig {
    name: String,
    namespace: String,
}

pub fn generate(input_config: InputConfig) -> MistResult {
    MistResultFiles::new()
        .add_file("deployment.yaml".to_string(), formatdoc!{"
            ---
            apiVersion: apps/v1
            kind: Deployment
            metadata:
              name: {0}
              namespace: {1}
              labels:
                app: {0}
            spec:
              replicas: 1
              selector:
                matchLabels:
                  app: {0}
              template:
                metadata:
                  labels:
                    app: {0}
                spec:
                  containers:
                  - image: nginx
                    name: nginx
                    ports:
                    - name: http
                      containerPort: 80
        ", input_config.name, input_config.namespace})
        .add_file("service.yaml".to_string(), formatdoc!{"
            ---
            apiVersion: v1
            kind: Service
            metadata:
              name: {0}
              namespace: {1}
              labels:
                app: {0}
            spec:
              selector:
                app: {0}
              ports:
              - name: http
                port: 80
                protocol: TCP
                targetPort: http
        ", input_config.name, input_config.namespace})
        .into()
}
