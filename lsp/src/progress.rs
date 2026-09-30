//! Optional, server-initiated startup progress. Never blocks evaluation.
use std::time::Instant;

use anyhow::Result;
use lsp_server::{Connection, Notification, Request, RequestId, Response};
use serde_json::{Value, json};

struct Task {
    due: Instant,
    requested: bool,
    begun: bool,
    message: String,
}

const TOKEN: &str = "zettyp/startup";

pub struct Progress {
    enabled: bool,
    started: bool,
    task: Option<Task>,
}

impl Progress {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            started: false,
            task: None,
        }
    }

    /// One startup task per server, spanning superseded evaluation attempts.
    /// Finishing (success, failure or shutdown) permanently silences this instance.
    pub fn start(&mut self, now: Instant) {
        if self.started {
            return;
        }
        self.started = true;
        if self.enabled {
            self.task = Some(Task {
                due: now,
                requested: false,
                begun: false,
                message: "Preparing evaluator and index".into(),
            });
        }
    }

    pub fn deadline(&self) -> Option<Instant> {
        self.task
            .as_ref()
            .filter(|task| !task.requested)
            .map(|task| task.due)
    }

    pub fn tick(&mut self, connection: &Connection, now: Instant) -> Result<()> {
        if let Some(task) = &mut self.task
            && !task.requested
            && now >= task.due
        {
            connection.sender.send(
                Request::new(
                    RequestId::from(TOKEN.to_owned()),
                    "window/workDoneProgress/create".into(),
                    json!({"token": TOKEN}),
                )
                .into(),
            )?;
            task.requested = true;
        }
        Ok(())
    }

    pub fn response(&mut self, connection: &Connection, response: &Response) -> Result<()> {
        let Some(task) = &mut self.task else {
            return Ok(());
        };
        if !task.requested || task.begun || response.id != RequestId::from(TOKEN.to_owned()) {
            return Ok(());
        }
        if response.error.is_some() {
            self.enabled = false;
            self.task = None;
        } else {
            notify(
                connection,
                json!({
                    "kind": "begin", "title": "Initializing knowledge index",
                    "message": task.message, "cancellable": false,
                }),
            )?;
            task.begun = true;
        }
        Ok(())
    }

    pub fn report(&mut self, connection: &Connection, message: &str) -> Result<()> {
        if let Some(task) = &mut self.task {
            task.message = message.to_owned();
            if task.begun {
                notify(connection, json!({"kind": "report", "message": message}))?;
            }
        }
        Ok(())
    }

    pub fn finish(&mut self, connection: &Connection, message: &str) -> Result<()> {
        if let Some(task) = self.task.take()
            && task.begun
        {
            notify(connection, json!({"kind": "end", "message": message}))?;
        }
        // Late create replies cannot resurrect this task. The token is never reused.
        Ok(())
    }
}

fn notify(connection: &Connection, value: Value) -> Result<()> {
    connection.sender.send(
        Notification::new("$/progress".into(), json!({"token": TOKEN, "value": value})).into(),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use lsp_server::Message;
    use std::time::Duration;

    fn create(client: &Connection) -> Request {
        let Message::Request(request) = client.receiver.try_recv().unwrap() else {
            panic!("expected create")
        };
        assert_eq!(request.method, "window/workDoneProgress/create");
        request
    }

    fn notification(client: &Connection, kind: &str) -> Value {
        let Message::Notification(n) = client.receiver.try_recv().unwrap() else {
            panic!("expected progress")
        };
        assert_eq!(n.method, "$/progress");
        assert_eq!(n.params["value"]["kind"], kind);
        assert!(n.params["value"].get("percentage").is_none());
        n.params
    }

    #[test]
    fn startup_waits_for_acceptance_and_balances() -> Result<()> {
        let (server, client) = Connection::memory();
        let mut progress = Progress::new(true);
        let now = Instant::now();
        progress.start(now);
        progress.tick(&server, now)?;
        let request = create(&client);
        progress.report(&server, "Validating saved index")?;
        assert!(client.receiver.is_empty());
        progress.response(&server, &Response::new_ok(request.id, ()))?;
        let begin = notification(&client, "begin");
        assert_eq!(begin["token"], request.params["token"]);
        assert_eq!(begin["value"]["cancellable"], false);
        assert_eq!(begin["value"]["message"], "Validating saved index");
        progress.report(&server, "Installing announcements")?;
        notification(&client, "report");
        progress.finish(&server, "Knowledge index ready")?;
        notification(&client, "end");
        Ok(())
    }

    #[test]
    fn retries_share_startup_and_all_later_updates_are_silent() -> Result<()> {
        let (server, client) = Connection::memory();
        let mut progress = Progress::new(true);
        let now = Instant::now();
        progress.start(now);
        progress.tick(&server, now)?;
        let request = create(&client);
        progress.response(&server, &Response::new_ok(request.id, ()))?;
        notification(&client, "begin");
        progress.start(now);
        progress.tick(&server, now + Duration::from_secs(10))?;
        assert!(client.receiver.is_empty());
        progress.finish(&server, "Ready")?;
        notification(&client, "end");
        for _ in 0..3 {
            progress.start(now);
            progress.tick(&server, now + Duration::from_secs(10))?;
            progress.report(&server, "Updating")?;
            progress.finish(&server, "Ready")?;
        }
        assert!(progress.deadline().is_none());
        assert!(client.receiver.is_empty());
        Ok(())
    }

    #[test]
    fn unsupported_or_rejected_clients_stay_silent() -> Result<()> {
        let (server, client) = Connection::memory();
        let now = Instant::now();
        let mut unsupported = Progress::new(false);
        unsupported.start(now);
        unsupported.tick(&server, now)?;
        assert!(client.receiver.is_empty());
        let mut progress = Progress::new(true);
        progress.start(now);
        progress.tick(&server, now)?;
        let request = create(&client);
        progress.response(
            &server,
            &Response::new_err(request.id, -32601, "Unsupported".into()),
        )?;
        progress.start(now);
        progress.tick(&server, now)?;
        progress.report(&server, "Installing")?;
        progress.finish(&server, "Ready")?;
        assert!(client.receiver.is_empty());
        Ok(())
    }

    #[test]
    fn late_and_duplicate_replies_cannot_resurrect_startup() -> Result<()> {
        for accepted in [false, true] {
            let (server, client) = Connection::memory();
            let mut progress = Progress::new(true);
            let now = Instant::now();
            progress.start(now);
            progress.tick(&server, now)?;
            let reply = Response::new_ok(create(&client).id, ());
            if accepted {
                progress.response(&server, &reply)?;
                notification(&client, "begin");
                progress.response(&server, &reply)?;
                assert!(client.receiver.is_empty());
            }
            progress.finish(&server, "Server shutting down")?;
            if accepted {
                notification(&client, "end");
            }
            progress.start(now);
            progress.tick(&server, now)?;
            progress.response(&server, &reply)?;
            assert!(client.receiver.is_empty());
        }
        Ok(())
    }
}
