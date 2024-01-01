# **Namespace** Example

This is a super basic package that generates YAML for a Namespace based on the provided name of the installation:

```sh
mistctl generate my-namespace -p mistletoe/examples/namespace-example:0.1.2
```

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: my-namespace
```
