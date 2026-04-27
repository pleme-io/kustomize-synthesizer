# kustomize-synthesizer

> **★★★ CSE / Knowable Construction.** This repo operates under **Constructive Substrate Engineering** — canonical specification at [`pleme-io/theory/CONSTRUCTIVE-SUBSTRATE-ENGINEERING.md`](https://github.com/pleme-io/theory/blob/main/CONSTRUCTIVE-SUBSTRATE-ENGINEERING.md). The Compounding Directive (operational rules: solve once, load-bearing fixes only, idiom-first, models stay current, direction beats velocity) is in the org-level pleme-io/CLAUDE.md ★★★ section. Read both before non-trivial changes.


Typed Kustomize manifest generation from proven types. Built on yaml-synthesizer. All output validated by tree-sitter-yaml.

## Tests: 27 | Status: Proven, tree-sitter Validated

## Core Types

| Type | Purpose |
|------|---------|
| `Kustomization` | Full kustomization.yaml: apiVersion, kind, namespace, resources, bases, patches, generators, images, labels, annotations, prefix/suffix |
| `Patch` | StrategicMerge / Json6902 / Inline |
| `ConfigMapGenerator` | Name, literals, files, behavior |
| `SecretGenerator` | Name, literals, type, behavior |
| `ImageOverride` | `retag(name, tag)` or `rename(name, new_name)` |

No Raw variant exists. Struct-based, not enum-based.

## Rendering

`render_kustomization(&Kustomization)` → kustomization.yaml as YamlNode

## tree-sitter Validation

5 tests validate kustomization output (basic, full, secrets, patches, labels) via `tree-sitter-yaml`.

## Dependencies

yaml-synthesizer (path). proptest, tree-sitter, tree-sitter-yaml (dev).
