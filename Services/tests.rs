// Copyright (C) 2026 slatkisasa
// SPDX-License-Identifier: AGPL-3.0-only
use crate::{account::Account, storage::Paths};
use std::fs;
fn fixture() -> (tempfile::TempDir, Paths) {
    let dir = tempfile::tempdir().unwrap();
    let paths = Paths {
        data: dir.path().join("metadata"),
        config: dir.path().join("fixture.json"),
        launcher: dir.path().join("launcher.exe"),
    };
    fs::write(&paths.config, br#"{"Client":{"SavedAccountNames":"first@example.invalid,SECOND@example.invalid","Other":true},"Unknown":{"large":123456789012345678901234567890}}"#).unwrap();
    (dir, paths)
}
#[test]
fn dotnet_metadata_roundtrip() {
    let (_dir, paths) = fixture();
    let json = r#"[{"Id":"d604de3a-7ab0-4a1a-9a01-4056a6606410","DisplayName":"日本語 🎮","AccountIdentifier":"first@example.invalid","CreatedAt":"2026-01-02T03:04:05.1234567+09:00","LastUsedAt":null}]"#;
    fs::create_dir_all(&paths.data).unwrap();
    fs::write(paths.data.join("accounts.json"), json).unwrap();
    let accounts = paths.load().unwrap();
    paths.save(&accounts).unwrap();
    let loaded = paths.load().unwrap();
    assert_eq!(loaded[0].display_name, "日本語 🎮");
    assert_eq!(loaded[0].id, accounts[0].id);
    assert!(loaded[0].last_used_at.is_none());
}
#[test]
fn preference_preserves_unknown_fields_and_exact_backup() {
    let (_dir, paths) = fixture();
    let before = fs::read(&paths.config).unwrap();
    let backup = paths
        .update_config(Some("second@example.invalid"), || Ok(()))
        .unwrap();
    assert_eq!(fs::read(backup).unwrap(), before);
    let after: serde_json::Value =
        serde_json::from_slice(&fs::read(&paths.config).unwrap()).unwrap();
    assert_eq!(after["Client"]["Other"], true);
    assert_eq!(
        after["Unknown"]["large"].to_string(),
        "123456789012345678901234567890"
    );
    assert_eq!(
        paths.remembered().unwrap(),
        ["second@example.invalid", "first@example.invalid"]
    );
}
#[test]
fn running_launcher_prevents_write() {
    let (_dir, paths) = fixture();
    let before = fs::read(&paths.config).unwrap();
    assert!(
        paths
            .update_config(None, || anyhow::bail!("still running"))
            .is_err()
    );
    assert_eq!(fs::read(&paths.config).unwrap(), before);
}
#[test]
fn restart_before_replacement_preserves_original() {
    let (_dir, paths) = fixture();
    let before = fs::read(&paths.config).unwrap();
    let calls = std::cell::Cell::new(0);
    assert!(
        paths
            .update_config(None, || {
                calls.set(calls.get() + 1);
                anyhow::ensure!(calls.get() < 2, "restarted");
                Ok(())
            })
            .is_err()
    );
    assert_eq!(fs::read(&paths.config).unwrap(), before);
}
#[test]
fn invalid_config_is_never_replaced() {
    let (_dir, paths) = fixture();
    for bytes in [
        r#"{"Client":[]}"#,
        r#"{"Client":{"SavedAccountNames":123}}"#,
        "bad json",
    ] {
        fs::write(&paths.config, bytes).unwrap();
        assert!(paths.update_config(None, || Ok(())).is_err());
        assert_eq!(fs::read_to_string(&paths.config).unwrap(), bytes);
    }
}
#[test]
fn new_signin_and_retention() {
    let (_dir, paths) = fixture();
    for _ in 0..23 {
        paths.update_config(None, || Ok(())).unwrap();
    }
    assert!(paths.remembered().unwrap().is_empty());
    assert_eq!(
        fs::read_dir(paths.data.join("Backups")).unwrap().count(),
        20
    );
}
#[test]
fn storage_atomic_replacement() {
    let (_dir, paths) = fixture();
    paths
        .save(&[Account::new("synthetic@example.invalid".into())])
        .unwrap();
    paths.save(&[]).unwrap();
    assert!(paths.load().unwrap().is_empty());
}

#[test]
fn removed_accounts_stay_removed_after_restart() {
    let (_dir, paths) = fixture();
    let mut accounts = paths.load_or_import().unwrap();
    assert_eq!(accounts.len(), 2);
    let removed = accounts.remove(0).account_identifier;
    paths.save(&accounts).unwrap();
    // Existing metadata must load independently of the launcher config.
    fs::write(&paths.config, "invalid config").unwrap();
    let reloaded = paths.load_or_import().unwrap();
    assert_eq!(reloaded.len(), 1);
    assert!(!reloaded.iter().any(|a| a.account_identifier == removed));
    paths.save(&[]).unwrap();
    assert!(paths.load_or_import().unwrap().is_empty());
}

#[test]
fn first_use_empty_list_and_duplicate_imports_are_persisted() {
    let (_dir, paths) = fixture();
    fs::write(
        &paths.config,
        r#"{"Client":{"SavedAccountNames":"first@example.invalid,FIRST@example.invalid"}}"#,
    )
    .unwrap();
    assert_eq!(paths.load_or_import().unwrap().len(), 1);
    fs::remove_file(paths.data.join("accounts.json")).unwrap();
    fs::remove_file(&paths.config).unwrap();
    assert!(paths.load_or_import().unwrap().is_empty());
    assert!(paths.data.join("accounts.json").is_file());
}

#[test]
fn invalid_metadata_is_never_imported_or_overwritten() {
    let (_dir, paths) = fixture();
    fs::create_dir_all(&paths.data).unwrap();
    let file = paths.data.join("accounts.json");
    let valid = serde_json::to_value(Account::new("synthetic@example.invalid".into())).unwrap();
    let mut invalid = vec!["{".into(), "null".into(), "[null]".into(), "[{}]".into()];
    for (field, value) in [
        ("Id", serde_json::json!(uuid::Uuid::nil())),
        ("DisplayName", serde_json::json!("  ")),
        ("AccountIdentifier", serde_json::Value::Null),
        ("AccountIdentifier", serde_json::json!("a,b")),
        ("AccountIdentifier", serde_json::json!(" a@example.invalid")),
        ("AccountIdentifier", serde_json::json!("")),
    ] {
        let mut record = valid.clone();
        record[field] = value;
        invalid.push(serde_json::to_string(&vec![record]).unwrap());
    }
    invalid.push(serde_json::to_string(&vec![valid.clone(), valid.clone()]).unwrap());
    let mut duplicate = valid.clone();
    duplicate["Id"] = serde_json::json!(uuid::Uuid::new_v4());
    duplicate["AccountIdentifier"] = serde_json::json!("SYNTHETIC@example.invalid");
    invalid.push(serde_json::to_string(&vec![valid, duplicate]).unwrap());
    for json in invalid {
        fs::write(&file, &json).unwrap();
        let error = paths.load_or_import().err().expect("invalid data accepted");
        let damaged = error
            .downcast_ref::<crate::storage::DamagedMetadata>()
            .unwrap();
        assert_eq!(damaged.original, json.as_bytes());
        assert_eq!(fs::read(&file).unwrap(), json.as_bytes());
        assert_eq!(fs::read_dir(&paths.data).unwrap().count(), 1);
    }
}

#[test]
fn invalid_save_preserves_previous_metadata() {
    let (_dir, paths) = fixture();
    let accounts = paths.load_or_import().unwrap();
    let file = paths.data.join("accounts.json");
    let original = fs::read(&file).unwrap();
    let mut invalid = accounts.clone();
    invalid[0].display_name = " ".into();
    assert!(paths.save(&invalid).is_err());
    assert!(
        paths
            .save(&[accounts[0].clone(), accounts[0].clone()])
            .is_err()
    );
    assert_eq!(fs::read(file).unwrap(), original);
}

#[test]
fn recovery_preserves_exact_bytes_and_reset_survives_restart() {
    let (_dir, paths) = fixture();
    fs::create_dir_all(&paths.data).unwrap();
    let original = b"\xef\xbb\xbfbroken data\r\n";
    fs::write(paths.data.join("accounts.json"), original).unwrap();
    let backup = paths.recover_accounts(original).unwrap();
    assert!(
        backup
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("accounts.json.damaged-")
    );
    assert_eq!(fs::read(backup).unwrap(), original);
    assert!(paths.load_or_import().unwrap().is_empty());
}

#[test]
fn recovery_rejects_changed_data_and_io_errors_are_not_damage() {
    let (_dir, paths) = fixture();
    fs::create_dir_all(&paths.data).unwrap();
    let file = paths.data.join("accounts.json");
    fs::write(&file, "new contents").unwrap();
    assert!(paths.recover_accounts(b"old contents").is_err());
    assert_eq!(fs::read_to_string(&file).unwrap(), "new contents");
    assert_eq!(fs::read_dir(&paths.data).unwrap().count(), 1);
    fs::remove_file(&file).unwrap();
    fs::create_dir(&file).unwrap();
    let error = paths.load_or_import().err().unwrap();
    assert!(
        error
            .downcast_ref::<crate::storage::DamagedMetadata>()
            .is_none()
    );
}

#[test]
fn new_signin_without_any_remembered_accounts() {
    let (_dir, paths) = fixture();
    let original = br#"{"Client":{"Other":true},"Unknown":42}"#;
    fs::write(&paths.config, original).unwrap();
    let backup = paths.update_config(None, || Ok(())).unwrap();
    assert_eq!(fs::read(backup).unwrap(), original);
    assert!(paths.remembered().unwrap().is_empty());
    let after: serde_json::Value =
        serde_json::from_slice(&fs::read(&paths.config).unwrap()).unwrap();
    assert_eq!(after["Client"]["SavedAccountNames"], "");
    assert_eq!(after["Client"]["Other"], true);
}

#[cfg(windows)]
#[test]
fn external_file_lock_preserves_original_and_retry_succeeds() {
    use std::os::windows::fs::OpenOptionsExt;
    let (_dir, paths) = fixture();
    let original = fs::read(&paths.config).unwrap();
    let lock = fs::OpenOptions::new()
        .read(true)
        .share_mode(windows_sys::Win32::Storage::FileSystem::FILE_SHARE_READ)
        .open(&paths.config)
        .unwrap();
    assert!(paths.update_config(None, || Ok(())).is_err());
    assert_eq!(fs::read(&paths.config).unwrap(), original);
    assert_eq!(
        fs::read_dir(paths.config.parent().unwrap())
            .unwrap()
            .count(),
        2
    ); // Fixture and metadata directory; no staging file.
    drop(lock);
    paths.update_config(None, || Ok(())).unwrap();
    assert!(paths.remembered().unwrap().is_empty());
}
