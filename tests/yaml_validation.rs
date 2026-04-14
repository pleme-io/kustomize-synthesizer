/// Prove that kustomize-synthesizer output is valid YAML via tree-sitter.
use kustomize_synthesizer::*;
use yaml_synthesizer::emit_file;

fn validate_yaml(source: &str) {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_yaml::language().into())
        .expect("failed to set YAML language");
    let tree = parser.parse(source, None).expect("parser returned no tree");
    assert!(
        !tree.root_node().has_error(),
        "tree-sitter detected YAML parse error in:\n{}",
        &source[..500.min(source.len())]
    );
}

#[test]
fn basic_kustomization_valid_yaml() {
    let k = Kustomization::new()
        .namespace("default")
        .resource("deployment.yaml")
        .resource("service.yaml");
    validate_yaml(&emit_file(&render_kustomization(&k)));
}

#[test]
fn full_kustomization_valid_yaml() {
    let k = Kustomization::new()
        .namespace("observability")
        .resource("../../base")
        .patch(Patch::StrategicMerge("patch-resources.yaml".into()))
        .config_map(ConfigMapGenerator::new("vector-config").file("vector.yaml"))
        .image(ImageOverride::retag("timberio/vector", "0.41.1"))
        .label("app.kubernetes.io/part-of", "shinryu")
        .annotation("fluxcd.io/automated", "true");
    validate_yaml(&emit_file(&render_kustomization(&k)));
}

#[test]
fn kustomization_with_secret_generator_valid() {
    let k = Kustomization::new()
        .secret(SecretGenerator::new("db-creds").opaque().literal("username", "admin"));
    validate_yaml(&emit_file(&render_kustomization(&k)));
}

#[test]
fn kustomization_with_json_patch_valid() {
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
    validate_yaml(&emit_file(&render_kustomization(&k)));
}

#[test]
fn kustomization_with_labels_and_prefix_valid() {
    let k = Kustomization::new()
        .prefix("staging-")
        .label("env", "staging")
        .resource("deployment.yaml");
    validate_yaml(&emit_file(&render_kustomization(&k)));
}
