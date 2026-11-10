use correo_storage::legacy::LegacyProfile;
use correo_storage::migration::{recover_interrupted_rollback, MigrationApplier, MigrationPreview};
use correo_storage::StorageError;
use std::path::{Path, PathBuf};

fn fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(path)
}

fn legacy_preview() -> MigrationPreview {
    let profile = LegacyProfile::read_from(fixture("legacy_profile")).unwrap();
    MigrationPreview::from_legacy_profile(profile).unwrap()
}

fn seed_existing_target(target: &Path) {
    std::fs::create_dir_all(target.join("scripts")).unwrap();
    std::fs::write(
        target.join("config.json"),
        r#"{"connections":[],"settings":{"saved_locale":"before"}}"#,
    )
    .unwrap();
    std::fs::write(target.join("scripts/original.js"), "logger.info('before');").unwrap();
}

#[test]
fn migration_apply_rejects_nested_backup_root_before_copying() {
    let temp = tempfile::tempdir().unwrap();
    let target = temp.path().join("current");
    let backup_root = target.join("migration-backups");
    seed_existing_target(&target);

    let applier = MigrationApplier::with_backup_root(&target, &backup_root);
    let error = applier.apply_preview(&legacy_preview()).unwrap_err();

    assert!(matches!(
        &error,
        StorageError::MigrationRollbackSafety { .. }
    ));
    assert!(error.to_string().contains("backup destination"));
    assert!(!backup_root.exists());
    assert!(std::fs::read_to_string(target.join("config.json"))
        .unwrap()
        .contains("before"));
    assert!(target.join("scripts/original.js").exists());
}

#[test]
fn interrupted_rollback_moves_displaced_target_back() {
    let temp = tempfile::tempdir().unwrap();
    let target = temp.path().join("current");
    let displaced = temp.path().join(".current-rollback-previous-1-2");
    let staging = temp.path().join(".current-rollback-staging-1-3");
    seed_existing_target(&displaced);
    seed_existing_target(&staging);

    assert!(recover_interrupted_rollback(&target).unwrap());

    assert!(target.join("scripts/original.js").exists());
    assert!(!displaced.exists());
    assert!(!staging.exists());
    assert!(!recover_interrupted_rollback(&target).unwrap());
}

#[test]
fn interrupted_rollback_leaves_existing_target_alone() {
    let temp = tempfile::tempdir().unwrap();
    let target = temp.path().join("current");
    let displaced = temp.path().join(".current-rollback-previous-1-2");
    seed_existing_target(&target);
    seed_existing_target(&displaced);

    assert!(!recover_interrupted_rollback(&target).unwrap());

    assert!(displaced.exists());
}

#[test]
fn interrupted_rollback_refuses_to_pick_between_two_displaced_targets() {
    let temp = tempfile::tempdir().unwrap();
    let target = temp.path().join("current");
    seed_existing_target(&temp.path().join(".current-rollback-previous-1-2"));
    seed_existing_target(&temp.path().join(".current-rollback-previous-3-4"));

    let error = recover_interrupted_rollback(&target).unwrap_err();

    assert!(matches!(
        error,
        StorageError::MigrationRollbackSafety { .. }
    ));
    assert!(!target.exists());
}
