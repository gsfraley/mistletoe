use indoc::{formatdoc};
use serde::{Deserialize};
use serde_yaml;

#[derive(Deserialize)]
struct InputConfig {
    name: String,
    namespace: String,
}

pub fn generate(input: &str) -> String {
    let input_config: InputConfig = serde_yaml::from_str(input).unwrap();

    let output = formatdoc! {"
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
    ", input_config.name, input_config.namespace};

    serde_yaml::to_string(&MistOutput::Ok(output)).unwrap()
}
