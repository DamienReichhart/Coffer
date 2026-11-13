//! End-to-end tests against the 7-Zip installed on this machine.
//!
//! They are skipped (and say so) when no 7-Zip is found, and are required in
//! CI by setting `SEVENZIP_REQUIRED=1`, where every runner installs 7-Zip.

use std::path::{Path, PathBuf};

use coffer_core::command::{self, Dialect};
use coffer_core::listing::Tree;
use coffer_core::{
    ArchiveFormat, CompressOptions, Error, Event, ExtractOptions, HashAlgorithm, OverwriteMode, SevenZip, runner,
};
use futures_util::StreamExt;

async fn seven() -> Option<SevenZip> {
    match SevenZip::locate(None).await {
        Ok(seven) => Some(seven),
        Err(error) if std::env::var_os("SEVENZIP_REQUIRED").is_some() => {
            panic!("SEVENZIP_REQUIRED is set but 7-Zip was not found: {error}")
        }
        Err(error) => {
            eprintln!("skipping: {error}");
            None
        }
    }
}

fn sample(dir: &Path) -> PathBuf {
    let src = dir.join("src");
    std::fs::create_dir_all(src.join("sub")).expect("mkdir");
    std::fs::write(src.join("a.txt"), "hello\n").expect("write");
    std::fs::write(src.join("sub").join("naïve file.txt"), "é ü\n").expect("write");
    std::fs::write(src.join("sub").join("data.bin"), vec![7_u8; 200_000]).expect("write");
    src
}

async fn finish(seven: &SevenZip, args: command::Args) -> Result<coffer_core::Outcome, std::sync::Arc<Error>> {
    let mut stream = Box::pin(runner::run(seven.clone(), args, None));
    let mut last = None;
    while let Some(event) = stream.next().await {
        if let Event::Finished(result) = event {
            last = Some(result);
        }
    }
    last.expect("a Finished event")
}

#[tokio::test]
async fn create_list_test_extract_round_trip() {
    let Some(seven) = seven().await else { return };
    let dir = tempfile::tempdir().expect("tempdir");
    let src = sample(dir.path());
    let archive = dir.path().join("out.7z");
    let dialect = Dialect::of(&seven);

    let options = CompressOptions {
        format: ArchiveFormat::SevenZ,
        solid: true,
        password: Some("pässword".into()),
        encrypt_names: true,
        ..Default::default()
    };
    let outcome = finish(&seven, command::add(dialect, &archive, &[src], &options)).await;
    assert!(outcome.is_ok(), "{outcome:?}");

    // Encrypted names: listing needs the password.
    let err = runner::list(&seven, &archive, None).await.expect_err("needs a password");
    assert!(err.needs_password(), "{err:?}");
    let err = runner::list(&seven, &archive, Some("wrong")).await.expect_err("wrong password");
    assert!(matches!(err, Error::WrongPassword), "{err:?}");

    let listing = runner::list(&seven, &archive, Some("pässword")).await.expect("list");
    assert_eq!(listing.info.kind, "7z");
    assert_eq!(listing.file_count(), 3);
    let tree = Tree::new(&listing.entries);
    assert!(tree.get("src/sub/naïve file.txt").is_some());

    let tested = finish(&seven, command::test(dialect, &archive, Some("pässword"))).await;
    assert!(tested.is_ok(), "{tested:?}");

    let out = dir.path().join("out");
    let options = ExtractOptions {
        destination: out.clone(),
        overwrite: OverwriteMode::Overwrite,
        keep_paths: true,
        password: Some("pässword".into()),
        only: vec!["src/sub/naïve file.txt".into()],
    };
    let extracted = finish(&seven, command::extract(dialect, &archive, &options)).await;
    assert!(extracted.is_ok(), "{extracted:?}");
    let text = std::fs::read_to_string(out.join("src/sub/naïve file.txt")).expect("extracted file");
    assert_eq!(text, "é ü\n");
    assert!(!out.join("src/a.txt").exists());
}

#[tokio::test]
async fn zip_delete_and_rename() {
    let Some(seven) = seven().await else { return };
    let dir = tempfile::tempdir().expect("tempdir");
    let src = sample(dir.path());
    let archive = dir.path().join("out.zip");
    let dialect = Dialect::of(&seven);
    let options = CompressOptions { format: ArchiveFormat::Zip, ..Default::default() };
    finish(&seven, command::add(dialect, &archive, &[src], &options)).await.expect("create");

    finish(&seven, command::delete(dialect, &archive, &["src/a.txt".into()], None)).await.expect("delete");
    finish(&seven, command::rename(dialect, &archive, "src/sub/data.bin", "src/sub/renamed.bin", None))
        .await
        .expect("rename");
    let listing = runner::list(&seven, &archive, None).await.expect("list");
    let paths: Vec<&str> = listing.entries.iter().map(|e| e.path.as_str()).collect();
    assert!(!paths.contains(&"src/a.txt"), "{paths:?}");
    assert!(paths.contains(&"src/sub/renamed.bin"), "{paths:?}");
}

#[tokio::test]
async fn checksums() {
    let Some(seven) = seven().await else { return };
    let dir = tempfile::tempdir().expect("tempdir");
    let src = sample(dir.path());
    let file = src.join("a.txt");
    let args = command::hash(Dialect::of(&seven), &[file], &[HashAlgorithm::Crc32, HashAlgorithm::Sha256]);
    let outcome = finish(&seven, args).await.expect("hash");
    let files = coffer_core::hash::parse(&outcome.output).expect("parse");
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].digests[0].1, "363A3020");
    assert_eq!(files[0].digests[1].1, "5891b5b522d5df086d0ff0b110fbd9d21bb4fc7163af34d08286a2e846f6be03");
}

#[tokio::test]
async fn not_an_archive() {
    let Some(seven) = seven().await else { return };
    let dir = tempfile::tempdir().expect("tempdir");
    let file = dir.path().join("notes.txt");
    std::fs::write(&file, "just text").expect("write");
    let err = runner::list(&seven, &file, None).await.expect_err("not an archive");
    assert!(matches!(err, Error::UnsupportedArchive { .. }), "{err:?}");
}
