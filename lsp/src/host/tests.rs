use super::*;
use serde_json::json;

fn announcement(path: &str, content: &str) -> Value {
    json!({"host.write-file": [{"version": 1, "id": "titles", "path": path, "content": content}]})
}

fn prepare(root: &Path, value: &Value) -> Result<Writes> {
    Writes::prepare(value, root, Some(Path::new("cache")), std::iter::empty())
}

#[test]
fn publishes_replaces_and_skips_unchanged_content() -> Result<()> {
    let root = tempfile::tempdir()?;
    let target = root.path().join("cache/titles.toml");
    let value = announcement("cache/titles.toml", "title = '中文'\n");
    prepare(root.path(), &value)?.commit(root.path())?;
    let modified = target.metadata()?.modified()?;
    prepare(root.path(), &value)?.commit(root.path())?;
    assert_eq!(target.metadata()?.modified()?, modified);
    assert_eq!(std::fs::read_to_string(&target)?, "title = '中文'\n");
    prepare(root.path(), &announcement("cache/titles.toml", "new"))?.commit(root.path())?;
    assert_eq!(std::fs::read_to_string(target)?, "new");
    assert_eq!(std::fs::read_dir(root.path().join("cache"))?.count(), 1);
    Ok(())
}

#[test]
fn rejects_unauthorized_paths_and_open_documents() -> Result<()> {
    let root = tempfile::tempdir()?;
    for path in [
        "/tmp/out",
        "cache/../out",
        "cache/./out",
        "cache//out",
        "cache\\out",
        "note/a.typ",
        "cache",
        "",
    ] {
        assert!(
            prepare(root.path(), &announcement(path, "bad")).is_err(),
            "{path}"
        );
    }
    let value = announcement("cache/titles", "bad");
    assert!(Writes::prepare(&value, root.path(), None, std::iter::empty()).is_err());
    assert!(
        Writes::prepare(
            &value,
            root.path(),
            Some(Path::new("cache")),
            [PathBuf::from("cache/titles")].into_iter()
        )
        .is_err()
    );
    assert!(!root.path().join("cache").exists());
    Ok(())
}

#[test]
fn validates_entire_batch_before_writing() -> Result<()> {
    let root = tempfile::tempdir()?;
    let valid = announcement("cache/titles", "ok");
    let mut duplicate = valid.clone();
    duplicate["host.write-file"]
        .as_array_mut()
        .unwrap()
        .push(valid["host.write-file"][0].clone());
    assert!(prepare(root.path(), &duplicate).is_err());
    duplicate["host.write-file"][1]["id"] = json!("another");
    assert!(prepare(root.path(), &duplicate).is_err());
    for (field, value) in [
        ("version", json!(2)),
        ("id", json!("")),
        ("content", json!(42)),
        ("unknown", json!(true)),
    ] {
        let mut invalid = valid.clone();
        invalid["host.write-file"][0][field] = value;
        assert!(prepare(root.path(), &invalid).is_err());
    }
    assert!(!root.path().join("cache").exists());
    Ok(())
}

#[cfg(unix)]
#[test]
fn rejects_symlink_ancestors_and_targets() -> Result<()> {
    let root = tempfile::tempdir()?;
    let outside = tempfile::tempdir()?;
    let pending = prepare(root.path(), &announcement("cache/titles", "bad"))?;
    std::os::unix::fs::symlink(outside.path(), root.path().join("cache"))?;
    assert!(pending.commit(root.path()).is_err());
    assert!(!outside.path().join("titles").exists());
    assert!(prepare(root.path(), &announcement("cache/titles", "bad")).is_err());
    std::fs::remove_file(root.path().join("cache"))?;
    std::fs::create_dir(root.path().join("cache"))?;
    std::os::unix::fs::symlink(
        outside.path().join("absent"),
        root.path().join("cache/titles"),
    )?;
    assert!(prepare(root.path(), &announcement("cache/titles", "bad")).is_err());
    Ok(())
}
