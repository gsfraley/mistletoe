use mistletoe_api::v1alpha1::{MistResult, MistOutput};
use mistletoe_bind::mistletoe_headers;

use indoc::formatdoc;
use serde::Deserialize;

mistletoe_headers! {"
  name: nginx-example
  labels:
    mistletoe.dev/group: mistletoe-examples
"}

#[derive(Deserialize)]
pub struct NginxExampleInputs {
    name: String,
    namespace: String,
}

fn generate(inputs: NginxExampleInputs) -> MistResult {
    let name = inputs.name;
    let namespace = inputs.namespace;

    let output = MistOutput::new()
        .with_file("deployment.yaml".to_string(), formatdoc!{"
            ---
            apiVersion: apps/v1
            kind: Deployment
            metadata:
              name: {name}
              namespace: {namespace}
              labels:
                app: {name}
            spec:
              replicas: 1
              selector:
                matchLabels:
                  app: {name}
              template:
                metadata:
                  labels:
                    app: {name}
                spec:
                  containers:
                  - image: nginx
                    name: nginx
                    ports:
                    - name: http
                      containerPort: 80"})
        .with_file("service.yaml".to_string(), formatdoc!{"
            ---
            apiVersion: v1
            kind: Service
            metadata:
              name: {name}
              namespace: {namespace}
              labels:
                app: {name}
            spec:
              selector:
                app: {name}
              ports:
              - name: http
                port: 80
                protocol: TCP
                targetPort: http"});

    Ok(output)
}
