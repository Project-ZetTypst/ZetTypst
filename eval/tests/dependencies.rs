use anyhow::Result;
use std::{collections::BTreeMap, fs};
use typst::foundations::Dict;
use zettyp_eval::{Runtime, WorldOptions};

const PREFIX: &str =
    "#let announce(value) = [#metadata((tag: label(\"test\"), value: value))<eval.announcement>]\n";

fn fixture(body: &str) -> Result<(tempfile::TempDir, Runtime)> {
    let root = tempfile::tempdir()?;
    fs::write(root.path().join("main.typ"), format!("{PREFIX}{body}"))?;
    let runtime = Runtime::new_with_options(
        root.path(),
        WorldOptions {
            ignore_system_fonts: true,
            ..Default::default()
        },
    )?;
    Ok((root, runtime))
}

#[test]
fn records_exact_read_bytes_and_survives_warm_evaluation() -> Result<()> {
    let (root, mut runtime) = fixture("#announce(read(\"data.txt\"))")?;
    fs::write(root.path().join("data.txt"), "before")?;
    for _ in 0..3 {
        let result = runtime.evaluate("main.typ", Dict::new())?;
        assert!(result.result.output.is_ok());
        assert!(runtime.validates(&result.dependencies, &BTreeMap::new()));
        assert!(
            result
                .dependencies
                .contains_text(&root.path().canonicalize()?.join("data.txt"), "before")
        );
    }
    let result = runtime.latest().unwrap().clone();
    fs::write(root.path().join("data.txt"), "after!")?;
    // Same file size does not hide changed content. Old provenance is immutable.
    assert!(!runtime.validates(&result.dependencies, &BTreeMap::new()));
    fs::write(root.path().join("data.txt"), "before")?;
    assert!(runtime.validates(&result.dependencies, &BTreeMap::new()));
    fs::remove_file(root.path().join("data.txt"))?;
    assert!(!runtime.validates(&result.dependencies, &BTreeMap::new()));
    Ok(())
}

#[test]
fn binary_reads_are_dependencies_too() -> Result<()> {
    let (root, mut runtime) = fixture("#announce(read(\"data.bin\", encoding: none))")?;
    fs::write(root.path().join("data.bin"), [0, 255, 1, 254])?;
    let result = runtime.evaluate("main.typ", Dict::new())?;
    assert!(result.result.output.is_ok());
    assert!(runtime.validates(&result.dependencies, &BTreeMap::new()));
    fs::write(root.path().join("data.bin"), [0, 255, 1, 253])?;
    assert!(!runtime.validates(&result.dependencies, &BTreeMap::new()));
    Ok(())
}

#[test]
fn divergent_overlays_cannot_be_persisted_or_restore_disk_ranges() -> Result<()> {
    let (root, mut runtime) = fixture("#announce(read(\"data.txt\"))")?;
    fs::write(root.path().join("data.txt"), "disk")?;
    let disk = runtime.evaluate("main.typ", Dict::new())?;
    let different = BTreeMap::from([("data.txt".into(), "unsaved".into())]);
    assert!(!runtime.validates(&disk.dependencies, &different));
    let unsaved = runtime.evaluate_with_sources("main.typ", Dict::new(), different.clone())?;
    assert!(unsaved.result.output.is_ok());
    assert!(!runtime.validates(&unsaved.dependencies, &different));
    assert!(!runtime.validates(&unsaved.dependencies, &BTreeMap::new()));
    let identical = BTreeMap::from([("data.txt".into(), "disk".into())]);
    assert!(runtime.validates(&disk.dependencies, &identical));
    Ok(())
}

#[test]
fn missing_reads_clock_and_font_bytes_fail_closed() -> Result<()> {
    for (body, reason) in [
        ("#announce(read(\"missing.txt\"))", "failed"),
        ("#announce(datetime.today())", "time-dependent"),
        ("Rendered text", "font bytes"),
    ] {
        let (_root, mut runtime) = fixture(body)?;
        let result = runtime.evaluate("main.typ", Dict::new())?;
        assert!(
            result
                .dependencies
                .unsupported
                .as_deref()
                .unwrap_or("")
                .contains(reason),
            "{body}: {:?}",
            result.dependencies
        );
        assert!(!runtime.validates(&result.dependencies, &BTreeMap::new()));
    }
    Ok(())
}

#[test]
fn package_resolution_shadowing_invalidates_even_when_old_files_remain() -> Result<()> {
    let (root, _) = fixture("#import \"@preview/dependency-test:0.1.0\": value\n#announce(value)")?;
    let local = root.path().join("local");
    let cache = root.path().join("cache");
    let suffix = "preview/dependency-test/0.1.0";
    let package = cache.join(suffix);
    fs::create_dir_all(&package)?;
    fs::write(
        package.join("typst.toml"),
        "[package]\nname = \"dependency-test\"\nversion = \"0.1.0\"\nentrypoint = \"lib.typ\"\n",
    )?;
    fs::write(package.join("lib.typ"), "#let value = \"cached\"")?;
    let mut runtime = Runtime::new_with_options(
        root.path(),
        WorldOptions {
            ignore_system_fonts: true,
            package_path: Some(local.clone()),
            package_cache_path: Some(cache),
            ..Default::default()
        },
    )?;
    let result = runtime.evaluate("main.typ", Dict::new())?;
    assert!(result.result.output.is_ok(), "{:?}", result.result.output);
    assert!(runtime.validates(&result.dependencies, &BTreeMap::new()));
    fs::create_dir_all(local.join(suffix))?;
    assert!(!runtime.validates(&result.dependencies, &BTreeMap::new()));
    Ok(())
}

#[test]
fn wrong_font_book_is_rejected() -> Result<()> {
    let (_root, mut runtime) = fixture("#announce(\"ok\")")?;
    let result = runtime.evaluate("main.typ", Dict::new())?;
    let mut deps = result.dependencies.clone();
    deps.font_book = Some("different font inventory".into());
    assert!(!runtime.validates(&deps, &BTreeMap::new()));
    Ok(())
}
