//! A single, serial evaluator. Scheduling and source versions belong to the LSP.
use std::collections::BTreeMap;
use std::path::{Component, PathBuf};
use std::thread;

use anyhow::{Result, bail};
use crossbeam_channel::{Receiver, Sender, bounded, unbounded};
use lsp_server::{ErrorCode, ResponseError};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use typst::foundations::{Dict, Value as TypstValue};
use zettyp_eval::{Runtime, WorldOptions};

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvalParams {
    pub entry: PathBuf,
    #[serde(default)]
    pub inputs: BTreeMap<String, String>,
}

impl EvalParams {
    pub fn validate(&self) -> Result<()> {
        if self.entry.is_absolute()
            || self
                .entry
                .components()
                .any(|part| matches!(part, Component::ParentDir))
            || !self
                .entry
                .components()
                .any(|part| matches!(part, Component::Normal(_)))
        {
            bail!("entry must be a non-empty project-relative path without '..'");
        }
        Ok(())
    }
}

pub struct Job {
    pub params: EvalParams,
    pub sources: BTreeMap<PathBuf, String>,
}

#[derive(Serialize)]
pub struct Output {
    pub revision: u64,
    pub output: Value,
    pub warnings: Vec<String>,
}

pub struct Worker {
    pub jobs: Sender<Job>,
    pub results: Receiver<Result<Output, ResponseError>>,
}

impl Worker {
    pub fn start(root: PathBuf, options: WorldOptions) -> Result<Self> {
        let (jobs, requests) = unbounded::<Job>();
        let (completed, results) = unbounded();
        let (ready, started) = bounded(1);
        thread::Builder::new()
            .name("zettyp-eval".into())
            .spawn(move || {
                let mut runtime = match Runtime::new_with_options(root, options) {
                    Ok(runtime) => {
                        let _ = ready.send(Ok(()));
                        runtime
                    }
                    Err(error) => {
                        let _ = ready.send(Err(error));
                        return;
                    }
                };
                for job in requests {
                    if completed.send(evaluate(&mut runtime, job)).is_err() {
                        break;
                    }
                }
            })?;
        started.recv()??;
        Ok(Self { jobs, results })
    }
}

fn evaluate(runtime: &mut Runtime, job: Job) -> Result<Output, ResponseError> {
    let inputs = job
        .params
        .inputs
        .into_iter()
        .map(|(key, value)| (key.into(), TypstValue::Str(value.into())))
        .collect::<Dict>();
    let evaluation = runtime
        .evaluate_with_sources(job.params.entry, inputs, job.sources)
        .map_err(|error| failure(ErrorCode::InvalidParams, error.to_string()))?;
    let warnings: Vec<_> = evaluation
        .result
        .warnings
        .iter()
        .map(|warning| warning.message.to_string())
        .collect();
    let value = evaluation
        .result
        .output
        .as_ref()
        .map_err(|error| ResponseError {
            code: ErrorCode::RequestFailed as i32,
            message: format!("{error:#}"),
            data: Some(json!({"revision": evaluation.revision, "warnings": warnings})),
        })?;
    let output = serde_json::to_value(value)
        .map_err(|error| failure(ErrorCode::RequestFailed, error.to_string()))?;
    Ok(Output {
        revision: evaluation.revision,
        output,
        warnings,
    })
}

pub fn failure(code: ErrorCode, message: impl Into<String>) -> ResponseError {
    ResponseError {
        code: code as i32,
        message: message.into(),
        data: None,
    }
}
