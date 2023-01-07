# Getting Started with Comet on KIND

```shell
# Create a "WASM in KinD" Cluster
kind create cluster --config kind-config.yaml
# Apply the Comet CNI plugin
kubectl apply -f install.yaml
# Run the example
kubectl run nginx --image=nginx
```