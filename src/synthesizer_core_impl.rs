//! Conformance to [`synthesizer_core`] traits.
//!
//! Wave 2 of the compound-knowledge refactor: purely additive. No behavior
//! change to kustomize-synthesizer's existing APIs — this module only adds
//! trait impls that downstream generic code can consume.
//!
//! kustomize-synthesizer is **struct-based**: it has no root node enum.
//! `Kustomization` is the outermost type representing a complete Kustomize
//! artifact, and rendering flows through `render_kustomization(&Kustomization)
//! -> YamlNode` followed by `yaml_synthesizer::emit_file`. The trait impl
//! delegates through that chain, exposing `Kustomization` itself as the
//! single-variant `SynthesizerNode`. `variant_id` returns `0` because there
//! is exactly one variant: a complete Kustomization.

use crate::render::render_kustomization;
use crate::types::Kustomization;
use synthesizer_core::{NoRawAttestation, SynthesizerNode};
use yaml_synthesizer::emit_file;

impl SynthesizerNode for Kustomization {
    fn emit(&self, indent: usize) -> String {
        // A complete Kustomization renders via `render_kustomization` to a
        // `YamlNode`, which is then serialized to bytes by `emit_file`. The
        // `emit_file` output has no leading indent; we prepend
        // `indent_unit * indent` to every non-empty line to honor the trait's
        // indent contract non-trivially.
        let body = emit_file(&render_kustomization(self));
        if indent == 0 {
            return body;
        }
        let pad = Self::indent_unit().repeat(indent);
        let trailing_newline = body.ends_with('\n');
        let indented = body
            .lines()
            .map(|line| {
                if line.is_empty() {
                    String::new()
                } else {
                    format!("{pad}{line}")
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        if trailing_newline {
            format!("{indented}\n")
        } else {
            indented
        }
    }

    fn indent_unit() -> &'static str {
        "  "
    }

    fn variant_id(&self) -> u8 {
        // Struct-based synthesizer: `Kustomization` has exactly one variant.
        0
    }
}

impl NoRawAttestation for Kustomization {
    fn attestation() -> &'static str {
        "no Raw variant — typed surface exhausts Kustomize semantics by \
         construction. Kustomization is a struct composed of typed fields \
         (Vec<String> resources/bases, Vec<Patch> patches, Vec<ConfigMapGenerator>, \
         Vec<SecretGenerator>, Vec<ImageOverride>, Vec<(String, String)> labels/ \
         annotations, Option<String> namespace/prefix/suffix). Patch is an enum \
         with three typed variants (StrategicMerge, Json6902, Inline) — none \
         accept unstructured YAML. All rendering flows through \
         render_kustomization → YamlNode → yaml_synthesizer::emit_file; the \
         underlying YamlNode has its own NoRawAttestation (its deprecated Raw \
         variant is scheduled for removal and is never constructed inside \
         kustomize-synthesizer). tests/synthesizer_core_conformance.rs \
         ::no_raw_constructor_in_production_source scans src/ for Raw-variant \
         constructor patterns; any accidental reintroduction fails CI."
    }
}
