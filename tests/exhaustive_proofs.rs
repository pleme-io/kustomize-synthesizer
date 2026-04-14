use kustomize_synthesizer::*;
use yaml_synthesizer::emit_file;

// ── Structural proofs ───────────────────────────────────────────────

#[test]
fn kustomization_api_version() {
    let k = Kustomization::new();
    let out = emit_file(&render_kustomization(&k));
    assert!(out.contains("apiVersion: kustomize.config.k8s.io/v1beta1"));
}

#[test]
fn kustomization_kind() {
    let k = Kustomization::new();
    let out = emit_file(&render_kustomization(&k));
    assert!(out.contains("kind: Kustomization"));
}

#[test]
fn kustomization_namespace() {
    let k = Kustomization::new().namespace("observability");
    let out = emit_file(&render_kustomization(&k));
    assert!(out.contains("namespace: observability"));
}

#[test]
fn kustomization_resources() {
    let k = Kustomization::new()
        .resource("deployment.yaml")
        .resource("service.yaml")
        .resource("configmap.yaml");
    let out = emit_file(&render_kustomization(&k));
    assert!(out.contains("resources:"));
    assert!(out.contains("deployment.yaml"));
    assert!(out.contains("service.yaml"));
    assert!(out.contains("configmap.yaml"));
}

#[test]
fn kustomization_bases() {
    let k = Kustomization::new().base("../../base").base("../../common");
    let out = emit_file(&render_kustomization(&k));
    assert!(out.contains("bases:"));
    assert!(out.contains("../../base"));
}

#[test]
fn kustomization_patches_strategic_merge() {
    let k = Kustomization::new().patch(Patch::StrategicMerge("patch-replicas.yaml".into()));
    let out = emit_file(&render_kustomization(&k));
    assert!(out.contains("patches:"));
    assert!(out.contains("patch-replicas.yaml"));
}

#[test]
fn kustomization_patches_json6902() {
    let k = Kustomization::new().patch(Patch::Json6902 {
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
    });
    let out = emit_file(&render_kustomization(&k));
    assert!(out.contains("patches:"));
    assert!(out.contains("Deployment"));
    assert!(out.contains("replace"));
    assert!(out.contains("/spec/replicas"));
}

#[test]
fn kustomization_config_map_generator() {
    let k = Kustomization::new().config_map(
        ConfigMapGenerator::new("app-config")
            .literal("LOG_LEVEL", "info")
            .literal("PORT", "8080"),
    );
    let out = emit_file(&render_kustomization(&k));
    assert!(out.contains("configMapGenerator:"));
    assert!(out.contains("app-config"));
    assert!(out.contains("LOG_LEVEL=info"));
}

#[test]
fn kustomization_secret_generator() {
    let k = Kustomization::new().secret(
        SecretGenerator::new("db-creds")
            .opaque()
            .literal("username", "admin"),
    );
    let out = emit_file(&render_kustomization(&k));
    assert!(out.contains("secretGenerator:"));
    assert!(out.contains("db-creds"));
    assert!(out.contains("Opaque"));
}

#[test]
fn kustomization_images() {
    let k = Kustomization::new()
        .image(ImageOverride::retag("nginx", "1.25"))
        .image(ImageOverride::rename("old-img", "new-img"));
    let out = emit_file(&render_kustomization(&k));
    assert!(out.contains("images:"));
    assert!(out.contains("name: nginx"));
    assert!(out.contains("newTag:") && out.contains("1.25"));
    assert!(out.contains("newName: new-img"));
}

#[test]
fn kustomization_common_labels() {
    let k = Kustomization::new()
        .label("app", "my-app")
        .label("env", "production");
    let out = emit_file(&render_kustomization(&k));
    assert!(out.contains("commonLabels:"));
    assert!(out.contains("app: my-app"));
    assert!(out.contains("env: production"));
}

#[test]
fn kustomization_common_annotations() {
    let k = Kustomization::new().annotation("version", "v1.2.3");
    let out = emit_file(&render_kustomization(&k));
    assert!(out.contains("commonAnnotations:"));
    assert!(out.contains("version: v1.2.3"));
}

#[test]
fn kustomization_name_prefix() {
    let k = Kustomization::new().prefix("staging-");
    let out = emit_file(&render_kustomization(&k));
    assert!(out.contains("namePrefix: staging-"));
}

#[test]
fn kustomization_name_suffix() {
    let k = Kustomization::new().suffix("-v2");
    let out = emit_file(&render_kustomization(&k));
    assert!(out.contains("nameSuffix:") && out.contains("-v2"));
}

// ── Determinism proofs ──────────────────────────────────────────────

#[test]
fn kustomization_deterministic() {
    let k = Kustomization::new()
        .namespace("prod")
        .resource("deployment.yaml")
        .label("app", "test")
        .image(ImageOverride::retag("nginx", "latest"));
    let a = emit_file(&render_kustomization(&k));
    let b = emit_file(&render_kustomization(&k));
    assert_eq!(a, b);
}

// ── Realistic proofs ────────────────────────────────────────────────

#[test]
fn realistic_fluxcd_overlay() {
    let k = Kustomization::new()
        .namespace("observability")
        .resource("../../base")
        .patch(Patch::StrategicMerge("patch-resources.yaml".into()))
        .config_map(
            ConfigMapGenerator::new("vector-config")
                .file("vector.yaml"),
        )
        .image(ImageOverride::retag("timberio/vector", "0.41.1-distroless-libc"))
        .label("app.kubernetes.io/part-of", "shinryu")
        .annotation("fluxcd.io/automated", "true");

    let out = emit_file(&render_kustomization(&k));

    assert!(out.contains("kustomize.config.k8s.io/v1beta1"));
    assert!(out.contains("Kustomization"));
    assert!(out.contains("namespace: observability"));
    assert!(out.contains("resources:"));
    assert!(out.contains("patches:"));
    assert!(out.contains("configMapGenerator:"));
    assert!(out.contains("images:"));
    assert!(out.contains("commonLabels:"));
    assert!(out.contains("commonAnnotations:"));
}

#[test]
fn realistic_k8s_gitops_base() {
    let k = Kustomization::new()
        .resource("deployment.yaml")
        .resource("service.yaml")
        .resource("networkpolicy.yaml")
        .resource("pdb.yaml")
        .resource("hpa.yaml")
        .resource("servicemonitor.yaml")
        .label("app.kubernetes.io/managed-by", "kustomize");

    let out = emit_file(&render_kustomization(&k));

    // All 6 resources listed
    assert!(out.contains("deployment.yaml"));
    assert!(out.contains("service.yaml"));
    assert!(out.contains("networkpolicy.yaml"));
    assert!(out.contains("pdb.yaml"));
    assert!(out.contains("hpa.yaml"));
    assert!(out.contains("servicemonitor.yaml"));
}
