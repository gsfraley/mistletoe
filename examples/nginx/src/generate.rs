use std::collections::HashMap;
use mistletoe_api::v0_1::MistResult;

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

    let mut files = HashMap::new();
    files.insert("deployment.yaml".to_string(), formatdoc!{"
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
    ", input_config.name, input_config.namespace});
    files.insert("service.yaml".to_string(), formatdoc!{"
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
    ", input_config.name, input_config.namespace});

    serde_yaml::to_string(&MistResult::Ok { files }).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    use indoc::indoc;

    #[test]
    fn test_generate() {
        const INPUT: &'static str = indoc! {"
            name: my-nginx
            namespace: my-namespace
        "};
        const EXPECTED_OUTPUT: &'static str = indoc! {"
            apiVersion: v1alpha1
            kind: MistResult
            data:
              result: Ok
              files:
                deployment.yaml: |
                  ---
                  apiVersion: apps/v1
                  kind: Deployment
                  metadata:
                    name: my-nginx
                    namespace: my-namespace
                    labels:
                      app: my-nginx
                  spec:
                    replicas: 1
                    selector:
                      matchLabels:
                        app: my-nginx
                    template:
                      metadata:
                        labels:
                          app: my-nginx
                      spec:
                        containers:
                        - image: nginx
                          name: nginx
                          ports:
                          - name: http
                            containerPort: 80
                service.yaml: |
                  ---
                  apiVersion: v1
                  kind: Service
                  metadata:
                    name: my-nginx
                    namespace: my-namespace
                    labels:
                      app: my-nginx
                  spec:
                    selector:
                      app: my-nginx
                    ports:
                    - name: http
                      port: 80
                      protocol: TCP
                      targetPort: http
        "};
        let output = generate(INPUT);
        assert_eq!(EXPECTED_OUTPUT, output);
    }
}
