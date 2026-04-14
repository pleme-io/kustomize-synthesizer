/// Proven types for Kustomize manifests.

/// A complete kustomization.yaml structure.
#[derive(Debug, Clone, PartialEq)]
pub struct Kustomization {
    pub api_version: KustomizeApiVersion,
    pub kind: KustomizeKind,
    pub namespace: Option<String>,
    pub resources: Vec<String>,
    pub bases: Vec<String>,
    pub patches: Vec<Patch>,
    pub config_map_generators: Vec<ConfigMapGenerator>,
    pub secret_generators: Vec<SecretGenerator>,
    pub images: Vec<ImageOverride>,
    pub common_labels: Vec<(String, String)>,
    pub common_annotations: Vec<(String, String)>,
    pub name_prefix: Option<String>,
    pub name_suffix: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum KustomizeApiVersion {
    V1Beta1,
}

#[derive(Debug, Clone, PartialEq)]
pub enum KustomizeKind {
    Kustomization,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Patch {
    /// Strategic merge patch from a file path.
    StrategicMerge(String),
    /// JSON 6902 patch.
    Json6902 {
        target: PatchTarget,
        ops: Vec<JsonPatchOp>,
    },
    /// Inline patch content.
    Inline {
        target: Option<PatchTarget>,
        patch: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct PatchTarget {
    pub group: Option<String>,
    pub version: String,
    pub kind: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JsonPatchOp {
    pub op: JsonPatchAction,
    pub path: String,
    pub value: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum JsonPatchAction {
    Add,
    Remove,
    Replace,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConfigMapGenerator {
    pub name: String,
    pub literals: Vec<(String, String)>,
    pub files: Vec<String>,
    pub behavior: GeneratorBehavior,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SecretGenerator {
    pub name: String,
    pub literals: Vec<(String, String)>,
    pub files: Vec<String>,
    pub secret_type: Option<String>,
    pub behavior: GeneratorBehavior,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GeneratorBehavior {
    Create,
    Replace,
    Merge,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImageOverride {
    pub name: String,
    pub new_name: Option<String>,
    pub new_tag: Option<String>,
    pub digest: Option<String>,
}

// ── Constructors ────────────────────────────────────────────────────

impl Kustomization {
    #[must_use]
    pub fn new() -> Self {
        Self {
            api_version: KustomizeApiVersion::V1Beta1,
            kind: KustomizeKind::Kustomization,
            namespace: None,
            resources: Vec::new(),
            bases: Vec::new(),
            patches: Vec::new(),
            config_map_generators: Vec::new(),
            secret_generators: Vec::new(),
            images: Vec::new(),
            common_labels: Vec::new(),
            common_annotations: Vec::new(),
            name_prefix: None,
            name_suffix: None,
        }
    }

    #[must_use]
    pub fn namespace(mut self, ns: &str) -> Self {
        self.namespace = Some(ns.to_string());
        self
    }

    #[must_use]
    pub fn resource(mut self, path: &str) -> Self {
        self.resources.push(path.to_string());
        self
    }

    #[must_use]
    pub fn base(mut self, path: &str) -> Self {
        self.bases.push(path.to_string());
        self
    }

    #[must_use]
    pub fn patch(mut self, p: Patch) -> Self {
        self.patches.push(p);
        self
    }

    #[must_use]
    pub fn config_map(mut self, generator: ConfigMapGenerator) -> Self {
        self.config_map_generators.push(generator);
        self
    }

    #[must_use]
    pub fn secret(mut self, generator: SecretGenerator) -> Self {
        self.secret_generators.push(generator);
        self
    }

    #[must_use]
    pub fn image(mut self, img: ImageOverride) -> Self {
        self.images.push(img);
        self
    }

    #[must_use]
    pub fn label(mut self, key: &str, value: &str) -> Self {
        self.common_labels.push((key.to_string(), value.to_string()));
        self
    }

    #[must_use]
    pub fn annotation(mut self, key: &str, value: &str) -> Self {
        self.common_annotations
            .push((key.to_string(), value.to_string()));
        self
    }

    #[must_use]
    pub fn prefix(mut self, p: &str) -> Self {
        self.name_prefix = Some(p.to_string());
        self
    }

    #[must_use]
    pub fn suffix(mut self, s: &str) -> Self {
        self.name_suffix = Some(s.to_string());
        self
    }
}

impl Default for Kustomization {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigMapGenerator {
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            literals: Vec::new(),
            files: Vec::new(),
            behavior: GeneratorBehavior::Create,
        }
    }

    #[must_use]
    pub fn literal(mut self, key: &str, value: &str) -> Self {
        self.literals.push((key.to_string(), value.to_string()));
        self
    }

    #[must_use]
    pub fn file(mut self, path: &str) -> Self {
        self.files.push(path.to_string());
        self
    }
}

impl SecretGenerator {
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            literals: Vec::new(),
            files: Vec::new(),
            secret_type: None,
            behavior: GeneratorBehavior::Create,
        }
    }

    #[must_use]
    pub fn literal(mut self, key: &str, value: &str) -> Self {
        self.literals.push((key.to_string(), value.to_string()));
        self
    }

    #[must_use]
    pub fn opaque(mut self) -> Self {
        self.secret_type = Some("Opaque".to_string());
        self
    }
}

impl ImageOverride {
    #[must_use]
    pub fn retag(name: &str, new_tag: &str) -> Self {
        Self {
            name: name.to_string(),
            new_name: None,
            new_tag: Some(new_tag.to_string()),
            digest: None,
        }
    }

    #[must_use]
    pub fn rename(name: &str, new_name: &str) -> Self {
        Self {
            name: name.to_string(),
            new_name: Some(new_name.to_string()),
            new_tag: None,
            digest: None,
        }
    }
}

impl KustomizeApiVersion {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::V1Beta1 => "kustomize.config.k8s.io/v1beta1",
        }
    }
}

impl KustomizeKind {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Kustomization => "Kustomization",
        }
    }
}

impl GeneratorBehavior {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Replace => "replace",
            Self::Merge => "merge",
        }
    }
}

impl JsonPatchAction {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Add => "add",
            Self::Remove => "remove",
            Self::Replace => "replace",
        }
    }
}
