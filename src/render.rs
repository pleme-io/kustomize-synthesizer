use yaml_synthesizer::{YamlEntry, YamlNode};

use crate::types::*;

/// Render a kustomization.yaml from proven `Kustomization` type.
#[must_use]
pub fn render_kustomization(k: &Kustomization) -> YamlNode {
    let mut entries = vec![
        YamlEntry::new("apiVersion", YamlNode::str(k.api_version.as_str())),
        YamlEntry::new("kind", YamlNode::str(k.kind.as_str())),
    ];

    if let Some(ref ns) = k.namespace {
        entries.push(YamlEntry::new("namespace", YamlNode::str(ns)));
    }

    if !k.resources.is_empty() {
        entries.push(YamlEntry::new(
            "resources",
            YamlNode::Seq(k.resources.iter().map(|r| YamlNode::str(r)).collect()),
        ));
    }

    if !k.bases.is_empty() {
        entries.push(YamlEntry::new(
            "bases",
            YamlNode::Seq(k.bases.iter().map(|b| YamlNode::str(b)).collect()),
        ));
    }

    if !k.patches.is_empty() {
        let patch_nodes: Vec<YamlNode> = k.patches.iter().map(render_patch).collect();
        entries.push(YamlEntry::new("patches", YamlNode::Seq(patch_nodes)));
    }

    if !k.config_map_generators.is_empty() {
        let gen_nodes: Vec<YamlNode> = k
            .config_map_generators
            .iter()
            .map(render_config_map_generator)
            .collect();
        entries.push(YamlEntry::new(
            "configMapGenerator",
            YamlNode::Seq(gen_nodes),
        ));
    }

    if !k.secret_generators.is_empty() {
        let gen_nodes: Vec<YamlNode> = k
            .secret_generators
            .iter()
            .map(render_secret_generator)
            .collect();
        entries.push(YamlEntry::new(
            "secretGenerator",
            YamlNode::Seq(gen_nodes),
        ));
    }

    if !k.images.is_empty() {
        let img_nodes: Vec<YamlNode> = k.images.iter().map(render_image_override).collect();
        entries.push(YamlEntry::new("images", YamlNode::Seq(img_nodes)));
    }

    if !k.common_labels.is_empty() {
        let label_entries: Vec<YamlEntry> = k
            .common_labels
            .iter()
            .map(|(key, val)| YamlEntry::new(key, YamlNode::str(val)))
            .collect();
        entries.push(YamlEntry::new("commonLabels", YamlNode::Map(label_entries)));
    }

    if !k.common_annotations.is_empty() {
        let ann_entries: Vec<YamlEntry> = k
            .common_annotations
            .iter()
            .map(|(key, val)| YamlEntry::new(key, YamlNode::str(val)))
            .collect();
        entries.push(YamlEntry::new(
            "commonAnnotations",
            YamlNode::Map(ann_entries),
        ));
    }

    if let Some(ref prefix) = k.name_prefix {
        entries.push(YamlEntry::new("namePrefix", YamlNode::str(prefix)));
    }

    if let Some(ref suffix) = k.name_suffix {
        entries.push(YamlEntry::new("nameSuffix", YamlNode::str(suffix)));
    }

    YamlNode::Map(entries)
}

fn render_patch(p: &Patch) -> YamlNode {
    match p {
        Patch::StrategicMerge(path) => YamlNode::map(vec![("path", YamlNode::str(path))]),
        Patch::Json6902 { target, ops } => {
            let ops_nodes: Vec<YamlNode> = ops
                .iter()
                .map(|op| {
                    let mut op_entries = vec![
                        YamlEntry::new("op", YamlNode::str(op.op.as_str())),
                        YamlEntry::new("path", YamlNode::str(&op.path)),
                    ];
                    if let Some(ref val) = op.value {
                        op_entries.push(YamlEntry::new("value", YamlNode::str(val)));
                    }
                    YamlNode::Map(op_entries)
                })
                .collect();

            YamlNode::map(vec![
                ("target", render_patch_target(target)),
                ("patch", YamlNode::Seq(ops_nodes)),
            ])
        }
        Patch::Inline { target, patch } => {
            let mut entries = vec![YamlEntry::new("patch", YamlNode::str(patch))];
            if let Some(t) = target {
                entries.push(YamlEntry::new("target", render_patch_target(t)));
            }
            YamlNode::Map(entries)
        }
    }
}

fn render_patch_target(t: &PatchTarget) -> YamlNode {
    let mut entries = vec![
        YamlEntry::new("version", YamlNode::str(&t.version)),
        YamlEntry::new("kind", YamlNode::str(&t.kind)),
        YamlEntry::new("name", YamlNode::str(&t.name)),
    ];
    if let Some(ref group) = t.group {
        entries.insert(0, YamlEntry::new("group", YamlNode::str(group)));
    }
    YamlNode::Map(entries)
}

fn render_config_map_generator(g: &ConfigMapGenerator) -> YamlNode {
    let mut entries = vec![YamlEntry::new("name", YamlNode::str(&g.name))];

    if !g.literals.is_empty() {
        let lits: Vec<YamlNode> = g
            .literals
            .iter()
            .map(|(k, v)| YamlNode::str(&format!("{k}={v}")))
            .collect();
        entries.push(YamlEntry::new("literals", YamlNode::Seq(lits)));
    }

    if !g.files.is_empty() {
        let files: Vec<YamlNode> = g.files.iter().map(|f| YamlNode::str(f)).collect();
        entries.push(YamlEntry::new("files", YamlNode::Seq(files)));
    }

    if g.behavior != GeneratorBehavior::Create {
        entries.push(YamlEntry::new(
            "behavior",
            YamlNode::str(g.behavior.as_str()),
        ));
    }

    YamlNode::Map(entries)
}

fn render_secret_generator(g: &SecretGenerator) -> YamlNode {
    let mut entries = vec![YamlEntry::new("name", YamlNode::str(&g.name))];

    if let Some(ref st) = g.secret_type {
        entries.push(YamlEntry::new("type", YamlNode::str(st)));
    }

    if !g.literals.is_empty() {
        let lits: Vec<YamlNode> = g
            .literals
            .iter()
            .map(|(k, v)| YamlNode::str(&format!("{k}={v}")))
            .collect();
        entries.push(YamlEntry::new("literals", YamlNode::Seq(lits)));
    }

    if g.behavior != GeneratorBehavior::Create {
        entries.push(YamlEntry::new(
            "behavior",
            YamlNode::str(g.behavior.as_str()),
        ));
    }

    YamlNode::Map(entries)
}

fn render_image_override(img: &ImageOverride) -> YamlNode {
    let mut entries = vec![YamlEntry::new("name", YamlNode::str(&img.name))];

    if let Some(ref nn) = img.new_name {
        entries.push(YamlEntry::new("newName", YamlNode::str(nn)));
    }
    if let Some(ref nt) = img.new_tag {
        entries.push(YamlEntry::new("newTag", YamlNode::str(nt)));
    }
    if let Some(ref d) = img.digest {
        entries.push(YamlEntry::new("digest", YamlNode::str(d)));
    }

    YamlNode::Map(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use yaml_synthesizer::emit_file;

    #[test]
    fn kustomization_has_api_version() {
        let k = Kustomization::new().resource("deployment.yaml");
        let out = emit_file(&render_kustomization(&k));
        assert!(out.contains("apiVersion: kustomize.config.k8s.io/v1beta1"));
    }

    #[test]
    fn kustomization_has_kind() {
        let k = Kustomization::new();
        let out = emit_file(&render_kustomization(&k));
        assert!(out.contains("kind: Kustomization"));
    }

    #[test]
    fn kustomization_resources() {
        let k = Kustomization::new()
            .resource("deployment.yaml")
            .resource("service.yaml");
        let out = emit_file(&render_kustomization(&k));
        assert!(out.contains("resources:"));
        assert!(out.contains("deployment.yaml"));
        assert!(out.contains("service.yaml"));
    }

    #[test]
    fn kustomization_namespace() {
        let k = Kustomization::new().namespace("production");
        let out = emit_file(&render_kustomization(&k));
        assert!(out.contains("namespace: production"));
    }

    #[test]
    fn kustomization_image_override() {
        let k = Kustomization::new().image(ImageOverride::retag("nginx", "1.25"));
        let out = emit_file(&render_kustomization(&k));
        assert!(out.contains("images:"));
        assert!(out.contains("name: nginx"));
        assert!(out.contains("newTag:") && out.contains("1.25"));
    }
}
