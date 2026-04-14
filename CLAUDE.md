# kustomize-synthesizer

Typed Kustomize manifest generation from proven types. Built on yaml-synthesizer.

## Tests: 22 | Status: Proven

## Core Types

| Type | Purpose |
|------|---------|
| `Kustomization` | Full kustomization.yaml: apiVersion, kind, namespace, resources, bases, patches, generators, images, labels, annotations, prefix/suffix |
| `Patch` | StrategicMerge / Json6902 / Inline |
| `ConfigMapGenerator` | Name, literals, files, behavior |
| `SecretGenerator` | Name, literals, type, behavior |
| `ImageOverride` | Retag or rename container images |

## Rendering

`render_kustomization(&Kustomization)` → kustomization.yaml as YamlNode

## Dependencies

yaml-synthesizer (path dep). proptest (dev).
