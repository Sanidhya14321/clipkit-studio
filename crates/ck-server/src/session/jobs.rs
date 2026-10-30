//! The export queue (spec 8.6): exports run one at a time off the request thread, in the order
//! they were asked for, and report progress.

use std::sync::{Arc, Mutex};

use ck_core::{CkError, CkResult, ErrorCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::export::{ExportRequest, ExportResult};
use super::{Session, is_zero};

/// Finished jobs kept for reading.
const MAX_JOBS: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum JobState {
    Queued,
    Running,
    Done,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExportJob {
    pub id: String,
    pub state: JobState,
    /// 0.0 to 1.0.
    pub progress: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<ExportResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExportJobs {
    /// Newest first.
    pub jobs: Vec<ExportJob>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub omitted: usize,
}

#[derive(Default)]
pub struct Jobs {
    list: Mutex<(u64, Vec<(ExportJob, ExportRequest)>)>,
    /// Held while one export runs.
    running: Mutex<()>,
}

impl Jobs {
    fn edit(&self, id: &str, f: impl FnOnce(&mut ExportJob)) {
        if let Ok(mut l) = self.list.lock()
            && let Some((job, _)) = l.1.iter_mut().find(|(j, _)| j.id == id)
        {
            f(job);
        }
    }

    /// Reports render progress on the running job.
    pub fn progress(&self, p: f64) {
        if let Ok(mut l) = self.list.lock()
            && let Some((job, _)) = l.1.iter_mut().find(|(j, _)| j.state == JobState::Running)
        {
            job.progress = p.clamp(0.0, 1.0);
        }
    }

    fn next(&self) -> Option<(String, ExportRequest)> {
        let mut l = self.list.lock().ok()?;
        let (job, req) = l.1.iter_mut().find(|(j, _)| j.state == JobState::Queued)?;
        job.state = JobState::Running;
        Some((job.id.clone(), req.clone()))
    }
}

/// Runs the oldest queued export.
fn run_next(session: &Session) {
    let Ok(_turn) = session.jobs.running.lock() else {
        return;
    };
    let Some((id, req)) = session.jobs.next() else {
        return;
    };
    let outcome = session.export(&req);
    session.jobs.edit(&id, |job| match outcome {
        Ok(r) => {
            job.state = JobState::Done;
            job.progress = 1.0;
            job.result = Some(r);
        }
        Err(e) => {
            job.state = JobState::Failed;
            job.error = Some(e.to_string());
        }
    });
}

impl Session {
    /// `export --queue`: returns at once with the job.
    pub fn export_enqueue(self: &Arc<Self>, req: &ExportRequest) -> CkResult<ExportJob> {
        let job = {
            let mut l = self
                .jobs
                .list
                .lock()
                .map_err(|_| CkError::new(ErrorCode::Server, "job list poisoned"))?;
            l.0 += 1;
            let job = ExportJob {
                id: format!("export-{}", l.0),
                state: JobState::Queued,
                progress: 0.0,
                result: None,
                error: None,
            };
            l.1.push((job.clone(), req.clone()));
            while l.1.len() > MAX_JOBS {
                let Some(old) =
                    l.1.iter()
                        .position(|(j, _)| matches!(j.state, JobState::Done | JobState::Failed))
                else {
                    break;
                };
                l.1.remove(old);
            }
            job
        };
        let session = Arc::clone(self);
        std::thread::spawn(move || run_next(&session));
        Ok(job)
    }

    /// `export jobs`: read-only.
    pub fn export_jobs(&self) -> CkResult<ExportJobs> {
        let l = self
            .jobs
            .list
            .lock()
            .map_err(|_| CkError::new(ErrorCode::Server, "job list poisoned"))?;
        Ok(ExportJobs {
            jobs: l.1.iter().rev().map(|(j, _)| j.clone()).collect(),
            omitted: 0,
        })
    }
}
