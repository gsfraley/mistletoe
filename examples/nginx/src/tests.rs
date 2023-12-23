use super::*;

const EXPECTED_INFO: &'static str = concatdoc! {"
    apiVersion: mistletoe.fraley.dev/v1alpha1
    kind: MistletoePackage
    metadata:
      name: ", env!("CARGO_PKG_NAME"), "
    spec:
      entrypoints:
        mistletoe_generate: mistletoe_generate
        mistletoe_alloc: mistletoe_alloc
        mistletoe_free: mistletoe_free
"};

const INPUT: &'static str = indoc! {"
    name: my-nginx
    namespace: my-namespace
"};

const EXPECTED_OUTPUT: &'static str = indoc! {"
    type: Ok
    value: |
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

#[test]
fn test_mistletoe_info() {
    let [ptr, len] = unsafe { *mistletoe_info() };
    let info = unsafe { std::str::from_utf8(std::slice::from_raw_parts(ptr as *const u8, len)).unwrap() };
    assert_eq!(EXPECTED_INFO, info);
}

#[test]
fn test_generate() {
    let output = generate(INPUT);
    assert_eq!(EXPECTED_OUTPUT, output);
}

#[test]
fn test_mistletoe_generate() {
    let [ret_ptr, ret_len] = unsafe { *mistletoe_generate(INPUT.as_ptr(), INPUT.len()) };
    let output = unsafe { std::str::from_utf8(std::slice::from_raw_parts(ret_ptr as *const u8, ret_len as usize)).unwrap() };
    assert_eq!(EXPECTED_OUTPUT, output);
}
