# **Nginx** Example

This is a super basic module that generates some Nginx YAML from a couple input parameters (just name and namespace).  For example, here's some input and output:

```yaml
name: my-nginx
namespace: my-namespace
```

```yaml
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
```
