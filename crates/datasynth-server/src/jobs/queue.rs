//! Job queue for async generation with submit/poll/cancel pattern.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore};
use uuid::Uuid;

/// Unique identifier for a job.
pub type JobId = String;

/// Status of a job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    /// Job is waiting to be processed.
    Queued,
    /// Job is currently being processed.
    Running,
    /// Job has completed successfully.
    Completed,
    /// Job has failed.
    Failed,
    /// Job was cancelled.
    Cancelled,
}

/// Result of a completed job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResult {
    /// Output directory where results were written.
    pub output_directory: Option<String>,
    /// Number of records generated.
    pub records_generated: Option<usize>,
    /// Duration in seconds.
    pub duration_seconds: Option<f64>,
    /// Error message if failed.
    pub error: Option<String>,
    /// Run manifest ID if available.
    pub manifest_id: Option<String>,
}

/// Request to create a new job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRequest {
    /// Configuration YAML or JSON as a string.
    #[serde(default)]
    pub config: Option<String>,
    /// Use demo preset if no config specified.
    #[serde(default)]
    pub demo: bool,
    /// Random seed.
    #[serde(default)]
    pub seed: Option<u64>,
    /// Output directory override.
    #[serde(default)]
    pub output_directory: Option<String>,
}

/// A job entry in the queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobEntry {
    /// Unique job ID.
    pub id: JobId,
    /// Current status.
    pub status: JobStatus,
    /// Job request parameters.
    pub request: JobRequest,
    /// When the job was submitted.
    pub submitted_at: DateTime<Utc>,
    /// When processing started.
    pub started_at: Option<DateTime<Utc>>,
    /// When the job completed/failed.
    pub completed_at: Option<DateTime<Utc>>,
    /// Job result (available when completed or failed).
    pub result: Option<JobResult>,
}

/// Summary view of a job (for listing).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSummary {
    pub id: JobId,
    pub status: JobStatus,
    pub submitted_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl From<&JobEntry> for JobSummary {
    fn from(entry: &JobEntry) -> Self {
        Self {
            id: entry.id.clone(),
            status: entry.status.clone(),
            submitted_at: entry.submitted_at,
            started_at: entry.started_at,
            completed_at: entry.completed_at,
        }
    }
}

/// In-memory job queue with concurrency control.
pub struct JobQueue {
    jobs: RwLock<HashMap<JobId, JobEntry>>,
    concurrency_semaphore: Arc<Semaphore>,
    max_concurrent: usize,
}

impl JobQueue {
    /// Creates a new job queue with the specified concurrency limit.
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            jobs: RwLock::new(HashMap::new()),
            concurrency_semaphore: Arc::new(Semaphore::new(max_concurrent)),
            max_concurrent,
        }
    }

    /// Returns the max concurrent jobs setting.
    pub fn max_concurrent(&self) -> usize {
        self.max_concurrent
    }

    /// Submits a new job and returns its ID.
    pub async fn submit(&self, request: JobRequest) -> JobId {
        let id = Uuid::new_v4().to_string();
        let entry = JobEntry {
            id: id.clone(),
            status: JobStatus::Queued,
            request,
            submitted_at: Utc::now(),
            started_at: None,
            completed_at: None,
            result: None,
        };

        let mut jobs = self.jobs.write().await;
        jobs.insert(id.clone(), entry);
        id
    }

    /// Gets the current state of a job.
    pub async fn get(&self, id: &str) -> Option<JobEntry> {
        let jobs = self.jobs.read().await;
        jobs.get(id).cloned()
    }

    /// Lists all jobs.
    pub async fn list(&self) -> Vec<JobSummary> {
        let jobs = self.jobs.read().await;
        let mut summaries: Vec<_> = jobs.values().map(JobSummary::from).collect();
        summaries.sort_by(|a, b| b.submitted_at.cmp(&a.submitted_at));
        summaries
    }

    /// Attempts to cancel a queued job. Returns true if cancelled.
    pub async fn cancel(&self, id: &str) -> bool {
        let mut jobs = self.jobs.write().await;
        if let Some(entry) = jobs.get_mut(id) {
            if entry.status == JobStatus::Queued {
                entry.status = JobStatus::Cancelled;
                entry.completed_at = Some(Utc::now());
                return true;
            }
        }
        false
    }

    /// Marks a job as running.
    pub async fn mark_running(&self, id: &str) {
        let mut jobs = self.jobs.write().await;
        if let Some(entry) = jobs.get_mut(id) {
            entry.status = JobStatus::Running;
            entry.started_at = Some(Utc::now());
        }
    }

    /// Marks a job as completed.
    pub async fn mark_completed(&self, id: &str, result: JobResult) {
        let mut jobs = self.jobs.write().await;
        if let Some(entry) = jobs.get_mut(id) {
            entry.status = JobStatus::Completed;
            entry.completed_at = Some(Utc::now());
            entry.result = Some(result);
        }
    }

    /// Marks a job as failed.
    pub async fn mark_failed(&self, id: &str, error: String) {
        let mut jobs = self.jobs.write().await;
        if let Some(entry) = jobs.get_mut(id) {
            entry.status = JobStatus::Failed;
            entry.completed_at = Some(Utc::now());
            entry.result = Some(JobResult {
                output_directory: None,
                records_generated: None,
                duration_seconds: None,
                error: Some(error),
                manifest_id: None,
            });
        }
    }

    /// Returns a clone of the concurrency semaphore for job execution.
    pub fn semaphore(&self) -> Arc<Semaphore> {
        Arc::clone(&self.concurrency_semaphore)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_submit_and_get() {
        let queue = JobQueue::new(4);
        let id = queue
            .submit(JobRequest {
                config: None,
                demo: true,
                seed: Some(42),
                output_directory: None,
            })
            .await;

        let job = queue.get(&id).await.expect("job should exist");
        assert_eq!(job.status, JobStatus::Queued);
        assert!(job.request.demo);
    }

    #[tokio::test]
    async fn test_status_transitions() {
        let queue = JobQueue::new(4);
        let id = queue
            .submit(JobRequest {
                config: None,
                demo: true,
                seed: None,
                output_directory: None,
            })
            .await;

        // Queued -> Running
        queue.mark_running(&id).await;
        let job = queue.get(&id).await.unwrap();
        assert_eq!(job.status, JobStatus::Running);
        assert!(job.started_at.is_some());

        // Running -> Completed
        queue
            .mark_completed(
                &id,
                JobResult {
                    output_directory: Some("/tmp/output".to_string()),
                    records_generated: Some(1000),
                    duration_seconds: Some(5.0),
                    error: None,
                    manifest_id: Some("run-123".to_string()),
                },
            )
            .await;
        let job = queue.get(&id).await.unwrap();
        assert_eq!(job.status, JobStatus::Completed);
        assert!(job.completed_at.is_some());
        assert_eq!(job.result.unwrap().records_generated, Some(1000));
    }

    #[tokio::test]
    async fn test_cancel_queued_job() {
        let queue = JobQueue::new(4);
        let id = queue
            .submit(JobRequest {
                config: None,
                demo: true,
                seed: None,
                output_directory: None,
            })
            .await;

        assert!(queue.cancel(&id).await);
        let job = queue.get(&id).await.unwrap();
        assert_eq!(job.status, JobStatus::Cancelled);
    }

    #[tokio::test]
    async fn test_cannot_cancel_running_job() {
        let queue = JobQueue::new(4);
        let id = queue
            .submit(JobRequest {
                config: None,
                demo: true,
                seed: None,
                output_directory: None,
            })
            .await;

        queue.mark_running(&id).await;
        assert!(!queue.cancel(&id).await); // Can't cancel running
    }

    #[tokio::test]
    async fn test_list_jobs() {
        let queue = JobQueue::new(4);
        queue
            .submit(JobRequest {
                config: None,
                demo: true,
                seed: None,
                output_directory: None,
            })
            .await;
        queue
            .submit(JobRequest {
                config: None,
                demo: true,
                seed: None,
                output_directory: None,
            })
            .await;

        let jobs = queue.list().await;
        assert_eq!(jobs.len(), 2);
    }

    #[tokio::test]
    async fn test_mark_failed() {
        let queue = JobQueue::new(4);
        let id = queue
            .submit(JobRequest {
                config: None,
                demo: true,
                seed: None,
                output_directory: None,
            })
            .await;

        queue.mark_running(&id).await;
        queue.mark_failed(&id, "Out of memory".to_string()).await;

        let job = queue.get(&id).await.unwrap();
        assert_eq!(job.status, JobStatus::Failed);
        assert_eq!(job.result.unwrap().error, Some("Out of memory".to_string()));
    }

    #[tokio::test]
    async fn test_concurrency_semaphore() {
        let queue = JobQueue::new(2);
        let sem = queue.semaphore();
        assert_eq!(sem.available_permits(), 2);

        let _permit1 = sem.acquire().await.unwrap();
        assert_eq!(sem.available_permits(), 1);

        let _permit2 = sem.acquire().await.unwrap();
        assert_eq!(sem.available_permits(), 0);
    }
}
