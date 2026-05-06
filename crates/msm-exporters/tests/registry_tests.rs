use msm_exporters::{
    ExportCapabilities, ExportPlan, ExportRegistry, ExportRegistryError, ExportRequest,
    ExportTarget, ExportTargetKind,
};

#[test]
fn registry_rejects_duplicate_target_kinds() {
    let mut registry = ExportRegistry::default();
    registry
        .register(Box::new(StubTarget::new("telegram")))
        .unwrap();

    let error = registry
        .register(Box::new(StubTarget::new("telegram")))
        .unwrap_err();

    assert_eq!(
        error,
        ExportRegistryError::DuplicateTargetKind {
            kind: ExportTargetKind::new("telegram"),
        }
    );
}

#[test]
fn registry_lookup_and_capabilities_are_stable() {
    let mut registry = ExportRegistry::default();
    registry
        .register(Box::new(StubTarget::new("telegram")))
        .unwrap();
    registry
        .register(Box::new(StubTarget::new("morestickers")))
        .unwrap();

    let target = registry
        .target(&ExportTargetKind::new("telegram"))
        .expect("target should exist");
    let plan = target
        .plan(ExportRequest {
            pack_id: "pack_1".into(),
            target_id: "target_1".into(),
            options_json: "{}".into(),
        })
        .unwrap();

    assert_eq!(plan.target_kind, ExportTargetKind::new("telegram"));
    assert_eq!(
        registry
            .capabilities()
            .into_iter()
            .map(|capabilities| capabilities.kind)
            .collect::<Vec<_>>(),
        vec![
            ExportTargetKind::new("morestickers"),
            ExportTargetKind::new("telegram"),
        ]
    );
}

#[test]
fn capabilities_are_serializable_for_api_and_web_surfaces() {
    let capabilities = ExportCapabilities {
        kind: ExportTargetKind::new("telegram"),
        display_name: "Telegram".into(),
        supports_remote_publication: true,
        supports_media_conversion: true,
        requires_credentials: true,
    };

    let json = serde_json::to_string(&capabilities).unwrap();

    assert_eq!(
        json,
        r#"{"kind":"telegram","displayName":"Telegram","supportsRemotePublication":true,"supportsMediaConversion":true,"requiresCredentials":true}"#
    );
}

#[derive(Debug)]
struct StubTarget {
    kind: ExportTargetKind,
}

impl StubTarget {
    fn new(kind: &str) -> Self {
        Self {
            kind: ExportTargetKind::new(kind),
        }
    }
}

impl ExportTarget for StubTarget {
    fn kind(&self) -> ExportTargetKind {
        self.kind.clone()
    }

    fn capabilities(&self) -> ExportCapabilities {
        ExportCapabilities {
            kind: self.kind.clone(),
            display_name: self.kind.as_str().to_owned(),
            supports_remote_publication: self.kind.as_str() == "telegram",
            supports_media_conversion: self.kind.as_str() == "telegram",
            requires_credentials: self.kind.as_str() == "telegram",
        }
    }

    fn plan(&self, request: ExportRequest) -> msm_exporters::ExportResult<ExportPlan> {
        Ok(ExportPlan {
            target_kind: self.kind.clone(),
            pack_id: request.pack_id,
            steps: vec!["prepare".into(), "publish".into()],
        })
    }
}
