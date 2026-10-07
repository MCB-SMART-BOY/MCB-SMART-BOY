use std::{
    collections::BTreeSet,
    fs, io,
    path::{Component, Path, PathBuf},
};

use super::{
    ExportError, OWNERSHIP_FILE, invalid, io_error, validate_directory, validate_regular_file,
};

const MANIFEST_HEADER: &str = "MCB-STATIC-EXPORT-1\n";

pub(super) struct PreparedOutput<'a> {
    root: &'a Path,
    files: BTreeSet<PathBuf>,
}

impl<'a> PreparedOutput<'a> {
    pub(super) fn new(root: &'a Path) -> Self {
        Self {
            root,
            files: BTreeSet::new(),
        }
    }

    fn store(
        &mut self,
        relative: &Path,
        save: impl FnOnce(&Path) -> Result<(), ExportError>,
    ) -> Result<(), ExportError> {
        validate_relative(relative)?;
        let target = self.root.join(relative);
        if self.files.contains(relative)
            || relative
                .ancestors()
                .skip(1)
                .any(|parent| self.files.contains(parent))
            || self.files.iter().any(|file| file.starts_with(relative))
        {
            return Err(ExportError::Collision { path: target });
        }
        let parent = target
            .parent()
            .ok_or_else(|| invalid(relative, "output needs a parent directory"))?;
        fs::create_dir_all(parent).map_err(|e| io_error("create output directory", parent, e))?;
        save(&target)?;
        self.files.insert(relative.to_owned());
        Ok(())
    }

    pub(super) fn write(&mut self, relative: &Path, bytes: &[u8]) -> Result<(), ExportError> {
        self.store(relative, |target| {
            fs::write(target, bytes).map_err(|e| io_error("write output", target, e))
        })
    }

    pub(super) fn copy(&mut self, relative: &Path, source: &Path) -> Result<(), ExportError> {
        self.store(relative, |target| {
            fs::copy(source, target)
                .map_err(|e| io_error("copy static asset from source", source, e))?;
            Ok(())
        })
    }

    pub(super) fn finish(&self) -> Result<(), ExportError> {
        let mut manifest = String::from(MANIFEST_HEADER);
        for file in &self.files {
            manifest.push_str(&file.to_string_lossy().replace('\\', "/"));
            manifest.push('\n');
        }
        let path = self.root.join(OWNERSHIP_FILE);
        fs::write(&path, manifest).map_err(|e| io_error("write ownership manifest", &path, e))
    }
}

fn validate_relative(path: &Path) -> Result<(), ExportError> {
    if path.components().next().is_none()
        || path
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
        || path.to_string_lossy().chars().any(char::is_control)
        || path == Path::new(OWNERSHIP_FILE)
    {
        return Err(invalid(path, "unsafe output-relative path"));
    }
    Ok(())
}

fn collect_existing(
    root: &Path,
    base: &Path,
    files: &mut BTreeSet<PathBuf>,
    dirs: &mut BTreeSet<PathBuf>,
) -> Result<(), ExportError> {
    for entry in fs::read_dir(root).map_err(|e| io_error("list previous output", root, e))? {
        let entry = entry.map_err(|e| io_error("read previous output entry", root, e))?;
        let path = entry.path();
        let relative = path
            .strip_prefix(base)
            .map_err(|_| invalid(&path, "output escapes its root"))?;
        if relative != Path::new(OWNERSHIP_FILE) {
            validate_relative(relative)?;
        }
        let metadata = fs::symlink_metadata(&path)
            .map_err(|e| io_error("inspect previous output", &path, e))?;
        if metadata.file_type().is_dir() {
            dirs.insert(relative.to_owned());
            collect_existing(&path, base, files, dirs)?;
        } else {
            validate_regular_file(&path)?;
            files.insert(relative.to_owned());
        }
    }
    Ok(())
}

fn validate_previous(target: &Path) -> Result<(), ExportError> {
    validate_directory(target)?;
    let marker = target.join(OWNERSHIP_FILE);
    validate_regular_file(&marker)?;
    let manifest =
        fs::read_to_string(&marker).map_err(|e| io_error("read ownership manifest", &marker, e))?;
    let list = manifest
        .strip_prefix(MANIFEST_HEADER)
        .ok_or_else(|| invalid(&marker, "not an exporter-owned directory"))?;
    let mut expected = BTreeSet::new();
    for entry in list.lines() {
        let path = PathBuf::from(entry);
        validate_relative(&path)?;
        if !expected.insert(path) {
            return Err(invalid(&marker, "duplicate manifest entry"));
        }
    }
    if !list.ends_with('\n') {
        return Err(invalid(&marker, "incomplete ownership manifest"));
    }
    expected.insert(PathBuf::from(OWNERSHIP_FILE));
    let mut allowed_dirs = BTreeSet::new();
    for file in &expected {
        for parent in file
            .ancestors()
            .skip(1)
            .filter(|path| !path.as_os_str().is_empty())
        {
            allowed_dirs.insert(parent.to_owned());
        }
    }
    let mut actual = BTreeSet::new();
    let mut actual_dirs = BTreeSet::new();
    collect_existing(target, target, &mut actual, &mut actual_dirs)?;
    if actual != expected || actual_dirs != allowed_dirs {
        return Err(invalid(
            target,
            "previous output contains missing or unowned entries",
        ));
    }
    Ok(())
}

fn replace_output_with(
    run: &Path,
    stage: &Path,
    mut rename: impl FnMut(&Path, &Path) -> io::Result<()>,
) -> Result<PathBuf, ExportError> {
    validate_directory(run)?;
    let target = run.join("pages");
    match fs::symlink_metadata(&target) {
        Ok(_) => validate_previous(&target)?,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            rename(stage, &target).map_err(|e| io_error("install exported pages", &target, e))?;
            return Ok(target);
        }
        Err(e) => return Err(io_error("inspect previous output", &target, e)),
    }
    let backup = tempfile::Builder::new()
        .prefix(".pages-backup-")
        .tempdir_in(run)
        .map_err(|e| io_error("create replacement backup", run, e))?;
    let old = backup.path().join("old");
    rename(&target, &old).map_err(|e| io_error("move previous export to backup", &target, e))?;
    let Err(install) = rename(stage, &target) else {
        backup
            .close()
            .map_err(|e| io_error("remove old exported pages", &old, e))?;
        return Ok(target);
    };
    match rename(&old, &target) {
        Ok(()) => Err(ExportError::Replacement {
            target,
            retained_backup: None,
            install,
            rollback: None,
        }),
        Err(rollback) => {
            let retained_backup = backup.keep().join("old");
            Err(ExportError::Replacement {
                target,
                retained_backup: Some(retained_backup),
                install,
                rollback: Some(rollback),
            })
        }
    }
}

pub(super) fn replace_output(run: &Path, stage: &Path) -> Result<PathBuf, ExportError> {
    replace_output_with(run, stage, |source, target| fs::rename(source, target))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replace_output_rejects_unowned_directory_without_changes() {
        let run = tempfile::tempdir().unwrap();
        fs::create_dir(run.path().join("pages")).unwrap();
        fs::write(run.path().join("pages/user.txt"), "keep").unwrap();
        let stage = tempfile::tempdir_in(run.path()).unwrap();
        PreparedOutput::new(stage.path()).finish().unwrap();
        assert!(replace_output(run.path(), stage.path()).is_err());
        assert_eq!(
            fs::read_to_string(run.path().join("pages/user.txt")).unwrap(),
            "keep"
        );
    }

    #[test]
    fn replace_output_removes_stale_page_and_installs_new_page() {
        let run = tempfile::tempdir().unwrap();
        let previous = run.path().join("pages");
        fs::create_dir(&previous).unwrap();
        let mut old = PreparedOutput::new(&previous);
        old.write(Path::new("writing/removed/index.html"), b"old")
            .unwrap();
        old.finish().unwrap();
        let stage = tempfile::tempdir_in(run.path()).unwrap();
        let mut fresh = PreparedOutput::new(stage.path());
        fresh.write(Path::new("index.html"), b"new").unwrap();
        fresh.finish().unwrap();
        replace_output(run.path(), stage.path()).unwrap();
        assert_eq!(fs::read(previous.join("index.html")).unwrap(), b"new");
        assert!(!previous.join("writing/removed/index.html").exists());
    }

    #[test]
    fn replace_output_failed_install_restores_previous_output() {
        let run = tempfile::tempdir().unwrap();
        let previous = run.path().join("pages");
        fs::create_dir(&previous).unwrap();
        let mut old = PreparedOutput::new(&previous);
        old.write(Path::new("index.html"), b"old").unwrap();
        old.finish().unwrap();
        let stage = tempfile::tempdir_in(run.path()).unwrap();
        PreparedOutput::new(stage.path()).finish().unwrap();
        let mut call = 0;
        let result = replace_output_with(run.path(), stage.path(), |from, to| {
            call += 1;
            if call == 2 {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "injected install failure",
                ));
            }
            fs::rename(from, to)
        });
        assert!(matches!(
            result,
            Err(ExportError::Replacement { rollback: None, .. })
        ));
        assert_eq!(fs::read(previous.join("index.html")).unwrap(), b"old");
    }

    #[test]
    fn replace_output_failed_rollback_retains_previous_output_backup() {
        let run = tempfile::tempdir().unwrap();
        let previous = run.path().join("pages");
        fs::create_dir(&previous).unwrap();
        let mut old = PreparedOutput::new(&previous);
        old.write(Path::new("index.html"), b"old").unwrap();
        old.finish().unwrap();
        let stage = tempfile::tempdir_in(run.path()).unwrap();
        PreparedOutput::new(stage.path()).finish().unwrap();
        let mut call = 0;
        let error = replace_output_with(run.path(), stage.path(), |from, to| {
            call += 1;
            if call >= 2 {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "injected rename failure",
                ));
            }
            fs::rename(from, to)
        })
        .unwrap_err();
        let ExportError::Replacement {
            retained_backup: Some(backup),
            rollback: Some(_),
            ..
        } = error
        else {
            panic!("expected retained backup after rollback failure");
        };
        assert_eq!(fs::read(backup.join("index.html")).unwrap(), b"old");
        fs::remove_dir_all(backup.parent().unwrap()).unwrap();
    }

    #[test]
    fn replace_output_rejects_unlisted_directory() {
        let run = tempfile::tempdir().unwrap();
        let previous = run.path().join("pages");
        fs::create_dir(&previous).unwrap();
        PreparedOutput::new(&previous).finish().unwrap();
        fs::create_dir(previous.join("user-work")).unwrap();
        let stage = tempfile::tempdir_in(run.path()).unwrap();
        PreparedOutput::new(stage.path()).finish().unwrap();
        assert!(replace_output(run.path(), stage.path()).is_err());
        assert!(previous.join("user-work").is_dir());
    }

    #[test]
    fn output_rejects_file_directory_collisions() {
        let root = tempfile::tempdir().unwrap();
        let mut output = PreparedOutput::new(root.path());
        output.write(Path::new("writing"), b"asset").unwrap();
        assert!(matches!(
            output.write(Path::new("writing/index.html"), b"page"),
            Err(ExportError::Collision { .. })
        ));
    }

    #[cfg(unix)]
    #[test]
    fn replace_output_rejects_symlink_in_previous_output() {
        use std::os::unix::fs::symlink;
        let run = tempfile::tempdir().unwrap();
        let previous = run.path().join("pages");
        fs::create_dir(&previous).unwrap();
        PreparedOutput::new(&previous).finish().unwrap();
        symlink("/", previous.join("escape")).unwrap();
        let stage = tempfile::tempdir_in(run.path()).unwrap();
        PreparedOutput::new(stage.path()).finish().unwrap();
        assert!(replace_output(run.path(), stage.path()).is_err());
    }
}
