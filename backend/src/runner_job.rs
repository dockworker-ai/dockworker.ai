use k8s_openapi::api::batch::v1::Job;
use k8s_openapi::api::core::v1::{
    Container, EmptyDirVolumeSource, PodSecurityContext, PodSpec, PodTemplateSpec,
    ResourceRequirements, SecurityContext, Volume, VolumeMount,
};
use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct BuildJobParams {
    pub build_id: String,
    pub tenant_id: String,
    pub git_repo: String,
    pub git_ref: String,
    pub dest_image: String,
    pub cache_repo: Option<String>,
}

pub fn create_runner_job_manifest(params: &BuildJobParams, namespace: &str) -> Job {
    let job_name = format!("dockworker-build-{}", params.build_id);

    let mut labels = BTreeMap::new();
    labels.insert(
        "app.kubernetes.io/name".to_string(),
        "dockworker-build".to_string(),
    );
    labels.insert(
        "app.kubernetes.io/component".to_string(),
        "build-runner".to_string(),
    );
    labels.insert(
        "dockworker.ai/build-id".to_string(),
        params.build_id.clone(),
    );
    labels.insert(
        "dockworker.ai/tenant-id".to_string(),
        params.tenant_id.clone(),
    );

    let mut buildctl_args = vec![
        "build".to_string(),
        "--frontend".to_string(),
        "dockerfile.v0".to_string(),
        "--local".to_string(),
        "context=/workspace".to_string(),
        "--local".to_string(),
        "dockerfile=/workspace".to_string(),
        "--output".to_string(),
        format!("type=image,name={},push=true", params.dest_image),
        "--progress".to_string(),
        "plain".to_string(),
    ];

    if let Some(ref cache) = params.cache_repo {
        buildctl_args.push("--export-cache".to_string());
        buildctl_args.push(format!("type=registry,ref={}", cache));
        buildctl_args.push("--import-cache".to_string());
        buildctl_args.push(format!("type=registry,ref={}", cache));
    }

    let mut requests = BTreeMap::new();
    requests.insert("cpu".to_string(), Quantity("1500m".to_string()));
    requests.insert("memory".to_string(), Quantity("2Gi".to_string()));
    requests.insert(
        "ephemeral-storage".to_string(),
        Quantity("10Gi".to_string()),
    );

    let mut limits = BTreeMap::new();
    limits.insert("cpu".to_string(), Quantity("2000m".to_string()));
    limits.insert("memory".to_string(), Quantity("4Gi".to_string()));
    limits.insert(
        "ephemeral-storage".to_string(),
        Quantity("18Gi".to_string()),
    );

    Job {
        metadata: ObjectMeta {
            name: Some(job_name),
            namespace: Some(namespace.to_string()),
            labels: Some(labels.clone()),
            ..Default::default()
        },
        spec: Some(k8s_openapi::api::batch::v1::JobSpec {
            active_deadline_seconds: Some(600),
            backoff_limit: Some(0),
            ttl_seconds_after_finished: Some(120),
            template: PodTemplateSpec {
                metadata: Some(ObjectMeta {
                    labels: Some(labels),
                    ..Default::default()
                }),
                spec: Some(PodSpec {
                    restart_policy: Some("Never".to_string()),
                    enable_service_links: Some(false),
                    automount_service_account_token: Some(false),
                    host_users: Some(false),
                    security_context: Some(PodSecurityContext {
                        run_as_user: Some(1000),
                        run_as_group: Some(1000),
                        fs_group: Some(1000),
                        run_as_non_root: Some(true),
                        ..Default::default()
                    }),
                    init_containers: Some(vec![Container {
                        name: "fetch-context".to_string(),
                        image: Some("alpine/git:2.47.2".to_string()),
                        command: Some(vec![
                            "git".to_string(),
                            "clone".to_string(),
                            "--depth".to_string(),
                            "1".to_string(),
                            "--branch".to_string(),
                            params.git_ref.clone(),
                            params.git_repo.clone(),
                            "/workspace".to_string(),
                        ]),
                        volume_mounts: Some(vec![VolumeMount {
                            name: "workspace".to_string(),
                            mount_path: "/workspace".to_string(),
                            ..Default::default()
                        }]),
                        security_context: Some(SecurityContext {
                            allow_privilege_escalation: Some(false),
                            read_only_root_filesystem: Some(true),
                            ..Default::default()
                        }),
                        resources: Some(ResourceRequirements {
                            requests: Some({
                                let mut m = BTreeMap::new();
                                m.insert("cpu".to_string(), Quantity("100m".to_string()));
                                m.insert("memory".to_string(), Quantity("128Mi".to_string()));
                                m
                            }),
                            limits: Some({
                                let mut m = BTreeMap::new();
                                m.insert("cpu".to_string(), Quantity("500m".to_string()));
                                m.insert("memory".to_string(), Quantity("256Mi".to_string()));
                                m
                            }),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }]),
                    containers: vec![Container {
                        name: "buildkit".to_string(),
                        image: Some("moby/buildkit:v0.26.2-rootless".to_string()),
                        args: Some(buildctl_args),
                        volume_mounts: Some(vec![
                            VolumeMount {
                                name: "workspace".to_string(),
                                mount_path: "/workspace".to_string(),
                                ..Default::default()
                            },
                            VolumeMount {
                                name: "buildkit-state".to_string(),
                                mount_path: "/home/user/.local/share/buildkit".to_string(),
                                ..Default::default()
                            },
                            VolumeMount {
                                name: "buildkit-run".to_string(),
                                mount_path: "/run/user/1000".to_string(),
                                ..Default::default()
                            },
                        ]),
                        resources: Some(ResourceRequirements {
                            requests: Some(requests),
                            limits: Some(limits),
                            ..Default::default()
                        }),
                        security_context: Some(SecurityContext {
                            allow_privilege_escalation: Some(false),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }],
                    volumes: Some(vec![
                        Volume {
                            name: "workspace".to_string(),
                            empty_dir: Some(EmptyDirVolumeSource {
                                size_limit: Some(Quantity("14Gi".to_string())),
                                ..Default::default()
                            }),
                            ..Default::default()
                        },
                        Volume {
                            name: "buildkit-state".to_string(),
                            empty_dir: Some(EmptyDirVolumeSource {
                                size_limit: Some(Quantity("16Gi".to_string())),
                                ..Default::default()
                            }),
                            ..Default::default()
                        },
                        Volume {
                            name: "buildkit-run".to_string(),
                            empty_dir: Some(EmptyDirVolumeSource {
                                medium: Some("Memory".to_string()),
                                size_limit: Some(Quantity("128Mi".to_string())),
                            }),
                            ..Default::default()
                        },
                    ]),
                    ..Default::default()
                }),
            },
            ..Default::default()
        }),
        status: None,
    }
}
