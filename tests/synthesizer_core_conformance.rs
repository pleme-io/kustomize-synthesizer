//! Integration tests proving `Kustomization` conforms to `synthesizer_core`
//! traits.
//!
//! Wave 2 of the compound-knowledge refactor. Every test calls one of
//! `synthesizer_core::node::laws::*` on a real `Kustomization` value,
//! compounding proof surface: the same laws prove properties of every
//! synthesizer that conforms.
//!
//! kustomize-synthesizer is **struct-based** (no root node enum).
//! `Kustomization` is the outermost type representing a complete Kustomize
//! artifact. `variant_id` returns 0 (single variant). The trait impl
//! delegates through `render_kustomization → YamlNode → emit_file`.

use kustomize_synthesizer::{
    ConfigMapGenerator, ImageOverride, JsonPatchAction, JsonPatchOp, Kustomization, Patch,
    PatchTarget, SecretGenerator,
};
use synthesizer_core::node::laws;
use synthesizer_core::{NoRawAttestation, SynthesizerNode};

// ─── Trait shape ────────────────────────────────────────────────────

#[test]
fn indent_unit_is_two_spaces() {
    assert_eq!(<Kustomization as SynthesizerNode>::indent_unit(), "  ");
}

#[test]
fn variant_id_is_zero_for_struct_based() {
    // Struct-based: Kustomization has exactly one variant — a complete
    // Kustomize artifact. variant_id is stable at 0.
    let k = Kustomization::new();
    assert_eq!(k.variant_id(), 0);
}

#[test]
fn variant_id_is_stable_across_configurations() {
    // All Kustomization instances share variant_id 0 — the struct itself is
    // the variant, regardless of field contents.
    let empty = Kustomization::new();
    let populated = Kustomization::new()
        .namespace("prod")
        .resource("deployment.yaml")
        .label("app", "test")
        .image(ImageOverride::retag("nginx", "latest"));
    assert_eq!(empty.variant_id(), populated.variant_id());
}

// ─── SynthesizerNode laws ───────────────────────────────────────────

#[test]
fn law_determinism_holds_on_empty_kustomization() {
    let k = Kustomization::new();
    assert!(laws::is_deterministic(&k, 0));
    assert!(laws::is_deterministic(&k, 3));
}

#[test]
fn law_determinism_holds_on_populated_kustomization() {
    let k = Kustomization::new()
        .namespace("observability")
        .resource("deployment.yaml")
        .resource("service.yaml")
        .label("app", "shinryu")
        .annotation("fluxcd.io/automated", "true")
        .image(ImageOverride::retag("nginx", "1.25"));
    assert!(laws::is_deterministic(&k, 0));
    assert!(laws::is_deterministic(&k, 2));
}

#[test]
fn law_determinism_holds_on_patches_and_generators() {
    let k = Kustomization::new()
        .patch(Patch::StrategicMerge("patch.yaml".into()))
        .patch(Patch::Json6902 {
            target: PatchTarget {
                group: Some("apps".into()),
                version: "v1".into(),
                kind: "Deployment".into(),
                name: "my-app".into(),
            },
            ops: vec![JsonPatchOp {
                op: JsonPatchAction::Replace,
                path: "/spec/replicas".into(),
                value: Some("3".into()),
            }],
        })
        .config_map(
            ConfigMapGenerator::new("app-config")
                .literal("LOG_LEVEL", "info")
                .literal("PORT", "8080"),
        )
        .secret(SecretGenerator::new("db-creds").opaque().literal("user", "admin"));
    assert!(laws::is_deterministic(&k, 0));
    assert!(laws::is_deterministic(&k, 1));
}

#[test]
fn law_honors_indent_unit_on_empty() {
    assert!(laws::honors_indent_unit(&Kustomization::new(), 0));
    assert!(laws::honors_indent_unit(&Kustomization::new(), 2));
}

#[test]
fn law_honors_indent_unit_on_populated() {
    let k = Kustomization::new()
        .namespace("prod")
        .resource("deployment.yaml");
    assert!(laws::honors_indent_unit(&k, 0));
    assert!(laws::honors_indent_unit(&k, 3));
}

#[test]
fn law_indent_monotone_len() {
    let k = Kustomization::new()
        .namespace("prod")
        .resource("deployment.yaml")
        .label("app", "test");
    assert!(laws::indent_monotone_len(&k, 0));
    assert!(laws::indent_monotone_len(&k, 2));
    assert!(laws::indent_monotone_len(&k, 5));
}

#[test]
fn law_variant_id_valid() {
    let samples = [
        Kustomization::new(),
        Kustomization::new().namespace("ns"),
        Kustomization::new().resource("x.yaml"),
        Kustomization::new().image(ImageOverride::retag("nginx", "1.0")),
        Kustomization::new().patch(Patch::StrategicMerge("p.yaml".into())),
    ];
    for k in &samples {
        assert!(laws::variant_id_is_valid(k));
    }
}

// ─── Emit sanity ────────────────────────────────────────────────────

#[test]
fn emit_at_indent_zero_matches_render_and_emit_file() {
    // Trait emit at indent 0 must match the direct render+emit_file pipeline
    // — no silent transformation at the default level.
    use kustomize_synthesizer::render_kustomization;
    use yaml_synthesizer::emit_file;

    let k = Kustomization::new()
        .namespace("default")
        .resource("deployment.yaml");
    let via_trait = k.emit(0);
    let via_direct = emit_file(&render_kustomization(&k));
    assert_eq!(via_trait, via_direct);
}

#[test]
fn emit_at_nonzero_indent_prepends_pad() {
    let k = Kustomization::new().namespace("default");
    let out = k.emit(2);
    // Every non-empty line should start with 4 spaces (indent_unit * 2).
    for line in out.lines() {
        if !line.is_empty() {
            assert!(
                line.starts_with("    "),
                "line does not honor indent: {line:?}"
            );
        }
    }
}

#[test]
fn emit_is_pure_over_many_calls() {
    let k = Kustomization::new()
        .namespace("prod")
        .resource("deployment.yaml")
        .label("app", "test");
    let first = k.emit(2);
    for _ in 0..32 {
        assert_eq!(k.emit(2), first);
    }
}

// ─── NoRawAttestation ───────────────────────────────────────────────

#[test]
fn attestation_is_nonempty() {
    assert!(!<Kustomization as NoRawAttestation>::attestation().is_empty());
}

#[test]
fn attestation_mentions_raw() {
    let s = <Kustomization as NoRawAttestation>::attestation();
    assert!(
        s.to_lowercase().contains("raw"),
        "attestation must explain how no-raw is enforced — got: {s}"
    );
}

#[test]
fn attestation_states_typed_surface_exhausts_semantics() {
    // The attestation contract for struct-based synthesizers: the typed
    // surface must exhaust the target language's semantics by construction
    // (no escape hatch exists at all).
    let s = <Kustomization as NoRawAttestation>::attestation();
    let lower = s.to_lowercase();
    assert!(
        lower.contains("typed") && lower.contains("kustomize"),
        "attestation must assert typed exhaustion of Kustomize semantics — got: {s}"
    );
}

// ─── No-raw source invariant ────────────────────────────────────────

#[test]
fn no_raw_constructor_in_production_source() {
    // Scan src/ for `Kustomization::Raw(...)`, `Self::Raw(...)`, or
    // `YamlNode::Raw(...)` constructor uses. Kustomization is struct-based
    // and has no Raw variant; YamlNode has a deprecated Raw variant we must
    // never construct from inside kustomize-synthesizer.
    //
    // Legitimate non-constructions (comments, attribute lines, match arms)
    // are exempted.
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut violations = Vec::new();
    for path in walk_rust_files(&src_dir) {
        let content = std::fs::read_to_string(&path).expect("read src file");
        let lines: Vec<&str> = content.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") || trimmed.starts_with("*") {
                continue;
            }
            // Match arms (patterns, not constructions).
            if line.contains("=>") {
                continue;
            }
            // Attribute lines.
            if trimmed.starts_with("#[") {
                continue;
            }
            // Preceding #[allow(deprecated)] → intentional reference.
            let prev_allows = i > 0 && lines[i - 1].contains("#[allow(deprecated)]");
            if prev_allows {
                continue;
            }
            if line.contains("Kustomization::Raw(")
                || line.contains("Self::Raw(")
                || line.contains("YamlNode::Raw(")
            {
                violations.push(format!("{}:{}", path.display(), i + 1));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "Raw construction in production source is forbidden \
         (Kustomization has no Raw variant; YamlNode::Raw is deprecated). \
         Violations: {violations:?}"
    );
}

fn walk_rust_files(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(root).expect("read src dir") {
        let entry = entry.expect("read dir entry");
        let path = entry.path();
        if path.is_dir() {
            out.extend(walk_rust_files(&path));
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            out.push(path);
        }
    }
    out
}
