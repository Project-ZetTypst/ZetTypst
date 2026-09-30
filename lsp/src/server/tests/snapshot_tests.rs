use super::*;
use crate::snapshot::Payload;
use zettyp_eval::content_hash;

fn payload(root: &Path) -> Payload {
    Payload {
        dependencies: serde_json::from_value(json!({
            "files": [{"path": root.join("main.typ"), "hash": content_hash(b"original")}],
            "packages": [], "font_book": null, "unsupported": null
        }))
        .unwrap(),
        output: json!({"lsp.hover": [{
            "applies-to": {"source": root.join("main.typ"), "range-utf16": {"start": {"line":0,"character":0}, "end":{"line":0,"character":8}}},
            "contents": {"kind":"plaintext", "value":"restored hover"}
        }]}),
        warnings: vec![],
    }
}

fn open(root: &Path, text: &str) -> Notification {
    Notification::new(
        "textDocument/didOpen".into(),
        json!({"textDocument": {
            "uri": Url::from_file_path(root.join("main.typ")).unwrap(),
            "languageId": "typst", "version":1, "text":text
        }}),
    )
}

fn hover(root: &Path) -> Request {
    Request::new(
        42.into(),
        "textDocument/hover".into(),
        json!({
            "textDocument":{"uri":Url::from_file_path(root.join("main.typ")).unwrap()},
            "position":{"line":0,"character":1}
        }),
    )
}

#[test]
fn snapshot_serves_hover_before_warmup_and_identical_open_preserves_it() -> Result<()> {
    let root = tempfile::tempdir()?;
    let root = root.path().canonicalize()?;
    std::fs::write(root.join("main.typ"), "original")?;
    let (connection, client) = Connection::memory();
    let (mut server, _jobs) = server(&connection);
    server.config.root = root.clone();
    server.config.client_root = root.clone();
    begin(&mut server, &client)?;
    server.worker_event(Event::Restored(payload(&root)))?;
    assert!(server.active.is_some(), "warmup still active");
    assert!(server.restored_dependencies.is_some());
    server.notification(open(&root, "original"))?;
    assert_eq!(server.generation, 0);
    progress_values(&client);
    server.request(hover(&root))?;
    let Message::Response(response) = client.receiver.try_recv()? else {
        panic!("query not answered")
    };
    assert_eq!(
        response.result.unwrap()["contents"]["value"],
        "restored hover"
    );
    server.completed(output())?;
    assert!(server.restored_dependencies.is_none());
    Ok(())
}

#[test]
fn different_buffer_or_unobserved_disk_change_invalidates_before_query() -> Result<()> {
    for disk in [false, true] {
        let dir = tempfile::tempdir()?;
        let root = dir.path().canonicalize()?;
        std::fs::write(root.join("main.typ"), "original")?;
        let (connection, client) = Connection::memory();
        let (mut server, _jobs) = server(&connection);
        server.config.root = root.clone();
        server.config.client_root = root.clone();
        server.watching = false;
        begin(&mut server, &client)?;
        server.worker_event(Event::Restored(payload(&root)))?;
        progress_values(&client);
        if disk {
            std::fs::write(root.join("main.typ"), "changed!")?;
        } else {
            server.notification(open(&root, "unsaved"))?;
        }
        server.request(hover(&root))?;
        assert!(server.cache.is_none());
        assert!(server.restored_dependencies.is_none());
        assert!(server.waiting.contains_key(&RequestId::from(42)));
        assert!(client.receiver.try_iter().all(|message| match message {
            Message::Response(_) => false,
            Message::Notification(n) => n.method != "$/progress",
            _ => true,
        }));
    }
    Ok(())
}

#[test]
fn late_snapshot_after_change_is_not_installed_and_error_drops_old_success() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let root = dir.path().canonicalize()?;
    std::fs::write(root.join("main.typ"), "original")?;
    let (connection, client) = Connection::memory();
    let (mut server, _jobs) = server(&connection);
    server.config.root = root.clone();
    server.config.client_root = root.clone();
    begin(&mut server, &client)?;
    server.changed()?;
    server.worker_event(Event::Restored(payload(&root)))?;
    assert!(server.cache.is_none());
    server.completed(output())?;
    progress_values(&client);
    server.due = Some(Instant::now());
    server.schedule()?;
    server.progress.tick(&connection, Instant::now())?;
    assert!(client.receiver.is_empty());
    server.worker_event(Event::Restored(payload(&root)))?;
    assert!(server.cache.as_ref().unwrap().is_ok());
    server.completed(Err(failure(
        ErrorCode::RequestFailed,
        "fresh compilation failed",
    )))?;
    assert!(server.cache.as_ref().unwrap().is_err());
    assert!(server.restored_dependencies.is_none());
    Ok(())
}
