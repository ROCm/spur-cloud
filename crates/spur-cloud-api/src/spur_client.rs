use std::collections::HashMap;

use tonic::transport::Channel;
use tracing::debug;

use spur_proto::proto::slurm_controller_client::SlurmControllerClient;
use spur_proto::proto::*;

/// Submit a GPU session as a Spur job. Returns the assigned job ID.
pub async fn submit_session(
    client: &mut SlurmControllerClient<Channel>,
    name: &str,
    gpu_type: &str,
    gpu_count: i32,
    container_image: &str,
    partition: Option<&str>,
    ssh_enabled: bool,
    time_limit_min: i32,
    session_id: &str,
    ssh_keys: &str,
) -> anyhow::Result<u32> {
    let mut environment = HashMap::new();
    environment.insert("GPUAAS_SESSION_ID".into(), session_id.to_string());
    if ssh_enabled && !ssh_keys.is_empty() {
        environment.insert("GPUAAS_SSH_KEYS".into(), ssh_keys.to_string());
    }

    let script = if ssh_enabled {
        // Entrypoint starts sshd then sleeps
        concat!(
            "#!/bin/bash\n",
            "mkdir -p /root/.ssh && chmod 700 /root/.ssh\n",
            "if [ -n \"$GPUAAS_SSH_KEYS\" ]; then\n",
            "  echo \"$GPUAAS_SSH_KEYS\" > /root/.ssh/authorized_keys\n",
            "  chmod 600 /root/.ssh/authorized_keys\n",
            "fi\n",
            "if command -v sshd >/dev/null 2>&1; then\n",
            "  mkdir -p /run/sshd\n",
            "  /usr/sbin/sshd -D &\n",
            "fi\n",
            "exec sleep infinity\n",
        )
        .to_string()
    } else {
        "#!/bin/bash\nexec sleep infinity\n".to_string()
    };

    let spec = JobSpec {
        name: name.to_string(),
        partition: partition.unwrap_or_default().to_string(),
        num_nodes: 1,
        num_tasks: 1,
        cpus_per_task: 8, // proportional: 8 CPUs per GPU
        gres: vec![format!("gpu:{}:{}", gpu_type, gpu_count)],
        script,
        environment,
        time_limit: Some(prost_types::Duration {
            seconds: time_limit_min as i64 * 60,
            nanos: 0,
        }),
        interactive: true,
        container_image: container_image.to_string(),
        ..Default::default()
    };

    let resp = client
        .submit_job(SubmitJobRequest { spec: Some(spec) })
        .await?;

    let job_id = resp.into_inner().job_id;
    debug!(job_id, name, "submitted session to spur");
    Ok(job_id)
}

/// Get job info from Spur.
pub async fn get_job(
    client: &mut SlurmControllerClient<Channel>,
    job_id: u32,
) -> anyhow::Result<Option<JobInfo>> {
    match client.get_job(GetJobRequest { job_id }).await {
        Ok(resp) => Ok(Some(resp.into_inner())),
        Err(e) if e.code() == tonic::Code::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Cancel a Spur job.
pub async fn cancel_job(
    client: &mut SlurmControllerClient<Channel>,
    job_id: u32,
) -> anyhow::Result<()> {
    client
        .cancel_job(CancelJobRequest {
            job_id,
            signal: 0,
            user: String::new(),
        })
        .await?;
    Ok(())
}

/// Get GPU capacity across all nodes.
pub async fn get_gpu_capacity(
    client: &mut SlurmControllerClient<Channel>,
) -> anyhow::Result<Vec<spur_cloud_common::gpu_types::GpuPool>> {
    use spur_cloud_common::gpu_types::{GpuNodeInfo, GpuPool};
    let resp = client
        .get_nodes(GetNodesRequest::default())
        .await?;

    let nodes = resp.into_inner().nodes;
    let mut pools: HashMap<String, GpuPool> = HashMap::new();

    for node in &nodes {
        let total_res = node.total_resources.as_ref();
        let alloc_res = node.alloc_resources.as_ref();

        if let Some(total) = total_res {
            for gpu in &total.gpus {
                let pool = pools.entry(gpu.gpu_type.clone()).or_insert_with(|| GpuPool {
                    gpu_type: gpu.gpu_type.clone(),
                    total: 0,
                    available: 0,
                    allocated: 0,
                    memory_mb: gpu.memory_mb,
                    nodes: Vec::new(),
                });
                pool.total += 1;
            }
        }

        if let Some(alloc) = alloc_res {
            for gpu in &alloc.gpus {
                if let Some(pool) = pools.get_mut(&gpu.gpu_type) {
                    pool.allocated += 1;
                }
            }
        }

        // Build per-node info
        let total_gpus = total_res.map(|r| r.gpus.len() as u32).unwrap_or(0);
        let alloc_gpus = alloc_res.map(|r| r.gpus.len() as u32).unwrap_or(0);

        if total_gpus > 0 {
            let gpu_type = total_res
                .and_then(|r| r.gpus.first())
                .map(|g| g.gpu_type.clone())
                .unwrap_or_default();

            if let Some(pool) = pools.get_mut(&gpu_type) {
                pool.nodes.push(GpuNodeInfo {
                    name: node.name.clone(),
                    total_gpus,
                    available_gpus: total_gpus.saturating_sub(alloc_gpus),
                    state: format!("{:?}", node.state()),
                });
            }
        }
    }

    // Compute available
    for pool in pools.values_mut() {
        pool.available = pool.total.saturating_sub(pool.allocated);
    }

    Ok(pools.into_values().collect())
}
