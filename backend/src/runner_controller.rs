use crate::runner_job::{create_runner_job_manifest, BuildJobParams};
use futures::{AsyncBufReadExt, Stream, TryStreamExt};
use k8s_openapi::api::batch::v1::Job;
use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::{Api, ListParams, LogParams, PostParams},
    Client,
};
use std::time::Duration;
use thiserror::Error;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{info, warn};

#[derive(Error, Debug)]
pub enum RunnerError {
    #[error("Kube API failure: {0}")]
    Kube(#[from] kube::Error),
    #[error("Pod timeout awaiting schedule")]
    PodWaitTimeout,
    #[error("IO streaming failure: {0}")]
    Io(#[from] std::io::Error),
}

pub struct BuildRunnerService {
    client: Client,
    namespace: String,
}

impl BuildRunnerService {
    pub fn new(client: Client, namespace: String) -> Self {
        Self { client, namespace }
    }

    /// Dispatches the Job and returns a receiver stream emitting log lines
    pub async fn execute_and_stream(
        &self,
        params: BuildJobParams,
    ) -> Result<impl Stream<Item = Result<String, RunnerError>>, RunnerError> {
        let jobs: Api<Job> = Api::namespaced(self.client.clone(), &self.namespace);
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), &self.namespace);

        let manifest = create_runner_job_manifest(&params, &self.namespace);
        let post_params = PostParams::default();
        let job = jobs.create(&post_params, &manifest).await?;
        let job_name = job.metadata.name.unwrap_or_default();
        info!(job = %job_name, build_id = %params.build_id, "Build Job submitted");

        let pod_name = self.await_active_pod(&pods, &params.build_id).await?;

        let (tx, rx) = mpsc::channel::<Result<String, RunnerError>>(128);
        let client_clone = self.client.clone();
        let ns_clone = self.namespace.clone();

        tokio::spawn(async move {
            let pods_api: Api<Pod> = Api::namespaced(client_clone, &ns_clone);
            let log_params = LogParams {
                container: Some("buildkit".to_string()),
                follow: true,
                tail_lines: Some(500),
                ..Default::default()
            };

            let mut stream_reader = None;
            for _ in 0..30 {
                match pods_api.log_stream(&pod_name, &log_params).await {
                    Ok(stream) => {
                        stream_reader = Some(stream);
                        break;
                    }
                    Err(e) => {
                        warn!(pod = %pod_name, "Waiting for buildkit container initialization: {e}");
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }

            if let Some(stream) = stream_reader {
                let mut lines = stream.lines();
                while let Ok(Some(line)) = lines.try_next().await {
                    if tx.send(Ok(line)).await.is_err() {
                        break;
                    }
                }
            } else {
                let _ = tx.send(Err(RunnerError::PodWaitTimeout)).await;
            }
        });

        Ok(ReceiverStream::new(rx))
    }

    async fn await_active_pod(&self, pods: &Api<Pod>, build_id: &str) -> Result<String, RunnerError> {
        let lp = ListParams::default()
            .labels(&format!("dockworker.ai/build-id={build_id}"))
            .timeout(60);

        for _ in 0..60 {
            let pod_list = pods.list(&lp).await?;
            if let Some(pod) = pod_list.items.into_iter().next() {
                if let Some(status) = pod.status {
                    if let Some(phase) = status.phase {
                        if phase == "Running" || phase == "Pending" || phase == "Succeeded" {
                            if let Some(name) = pod.metadata.name {
                                return Ok(name);
                            }
                        }
                    }
                }
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        Err(RunnerError::PodWaitTimeout)
    }
}
