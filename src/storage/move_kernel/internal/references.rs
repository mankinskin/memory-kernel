use super::*;

pub(in crate::storage::move_kernel) fn build_reference_visibility<D: MoveDomain + ?Sized>(
    domain: &D,
    target_store_root: &Path,
    target_store_present: bool,
    inbound: &BTreeSet<Uuid>,
    outbound: &BTreeSet<Uuid>,
) -> MoveResult<Vec<MoveReferenceVisibility>> {
    if !target_store_present {
        return Ok(Vec::new());
    }

    let mut visibility = Vec::new();
    for related_entity_id in inbound.iter().chain(outbound.iter()).copied() {
        let visible_from_destination =
            domain.entity_indexed_in(target_store_root, &related_entity_id)?;
        let direction = if outbound.contains(&related_entity_id) {
            MoveReferenceDirection::Outbound
        } else {
            MoveReferenceDirection::Inbound
        };
        visibility.push(MoveReferenceVisibility {
            related_entity_id,
            direction,
            visible_from_destination,
        });
    }

    Ok(visibility)
}

pub(in crate::storage::move_kernel) fn collect_plan_path_reference_files(
    source_entity_exists: bool,
    source_git_root: &Path,
    target_git_root: &Path,
    resolved_source_path: &Path,
    source_store_root: &Path,
    target_store_root: &Path,
    subdir: &str,
    blockers: &mut Vec<MoveBlocker>,
) -> Vec<PathBuf> {
    if !source_entity_exists {
        return Vec::new();
    }

    let mut files = BTreeSet::new();
    collect_candidate_reference_files(
        source_git_root,
        resolved_source_path,
        source_store_root,
        target_store_root,
        subdir,
        blockers,
        &mut files,
    );

    if source_git_root != target_git_root {
        collect_candidate_reference_files(
            target_git_root,
            resolved_source_path,
            source_store_root,
            target_store_root,
            subdir,
            blockers,
            &mut files,
        );
    }

    files.into_iter().collect()
}

pub(in crate::storage::move_kernel) fn collect_plan_path_reference_files_for_set(
    entity_paths: &BTreeMap<Uuid, PathBuf>,
    source_git_root: &Path,
    target_git_root: &Path,
    source_store_root: &Path,
    target_store_root: &Path,
    subdir: &str,
) -> MovePathReferenceSet {
    const GIT_GREP_PATTERN_CHUNK_SIZE: usize = 64;

    let mut result = MovePathReferenceSet::default();
    let mut repositories = vec![source_git_root];
    if source_git_root != target_git_root {
        repositories.push(target_git_root);
    }

    for git_root in repositories {
        let mut candidates = BTreeMap::<String, BTreeSet<Uuid>>::new();
        for (entity_id, entity_path) in entity_paths {
            let absolute = entity_path.to_string_lossy().replace('\\', "/");
            candidates.entry(absolute).or_default().insert(*entity_id);
            if let Ok(relative) = safe_strip_prefix(entity_path, git_root) {
                candidates
                    .entry(relative.to_string_lossy().replace('\\', "/"))
                    .or_default()
                    .insert(*entity_id);
            }
        }
        if candidates.is_empty() {
            continue;
        }

        let candidate_entries = candidates.iter().collect::<Vec<_>>();
        for candidate_chunk in candidate_entries.chunks(GIT_GREP_PATTERN_CHUNK_SIZE) {
            let mut command = Command::new("git");
            command.args([
                "-C",
                &git_root.to_string_lossy(),
                "grep",
                "-nF",
                "--full-name",
            ]);
            for (candidate, _) in candidate_chunk {
                command.args(["-e", candidate]);
            }
            command.arg("--");
            let output = match command.output() {
                Ok(output) => output,
                Err(error) => {
                    result
                        .blockers
                        .push(MoveBlocker::PathReferenceScanUnavailable {
                            reason: error.to_string(),
                        });
                    continue;
                }
            };
            if !output.status.success() && output.status.code() != Some(1) {
                result
                    .blockers
                    .push(MoveBlocker::PathReferenceScanUnavailable {
                        reason: String::from_utf8_lossy(&output.stderr).trim().to_string(),
                    });
                continue;
            }

            for line in String::from_utf8_lossy(&output.stdout).lines() {
                let Some((file, remainder)) = line.split_once(':') else {
                    continue;
                };
                let Some((_, content)) = remainder.split_once(':') else {
                    continue;
                };
                let candidate_file = git_root.join(file);
                if !is_persistent_move_reference_file(
                    &candidate_file,
                    source_store_root,
                    target_store_root,
                    subdir,
                ) {
                    continue;
                }
                for (candidate, entity_ids) in candidate_chunk {
                    if content.contains(*candidate) {
                        for entity_id in *entity_ids {
                            result
                                .files_by_entity
                                .entry(*entity_id)
                                .or_default()
                                .push(candidate_file.clone());
                        }
                    }
                }
            }
        }
    }

    for files in result.files_by_entity.values_mut() {
        files.sort();
        files.dedup();
    }
    result
}

pub(in crate::storage::move_kernel) fn collect_candidate_reference_files(
    git_root: &Path,
    resolved_source_path: &Path,
    source_store_root: &Path,
    target_store_root: &Path,
    subdir: &str,
    blockers: &mut Vec<MoveBlocker>,
    files: &mut BTreeSet<PathBuf>,
) {
    match git_tracked_path_reference_files(git_root, resolved_source_path) {
        Ok(found) => {
            for file in found {
                let candidate = git_root.join(file);
                if is_persistent_move_reference_file(
                    &candidate,
                    source_store_root,
                    target_store_root,
                    subdir,
                ) {
                    files.insert(candidate);
                }
            }
        }
        Err(reason) => blockers.push(MoveBlocker::PathReferenceScanUnavailable { reason }),
    }
}

pub(in crate::storage::move_kernel) fn restore_rewritten_path(
    rewrite: &MovePathRewrite,
) -> MoveResult<()> {
    if let Some(previous_content) = &rewrite.previous_content {
        fs::write(&rewrite.path, previous_content.as_bytes())?;
        return Ok(());
    }

    if !rewrite.replacements.is_empty() {
        let bytes = fs::read(&rewrite.path)?;
        let current_content = String::from_utf8(bytes).map_err(|_| {
            MoveError::Domain(format!(
                "rewritten file {} is not valid utf-8 for rollback",
                normalize_slashes(&rewrite.path)
            ))
        })?;

        let mut restored = current_content.clone();
        for replacement in rewrite.replacements.iter().rev() {
            restored = restored.replace(&replacement.after, &replacement.before);
        }
        fs::write(&rewrite.path, restored.as_bytes())?;
        return Ok(());
    }

    if path_buf_is_empty(&rewrite.repo_root) || path_buf_is_empty(&rewrite.repo_relative_path) {
        return Err(MoveError::Domain(format!(
            "journal rewrite record for {} is missing rollback metadata",
            normalize_slashes(&rewrite.path)
        )));
    }

    git_restore_tracked_path(&rewrite.repo_root, &rewrite.repo_relative_path)
}

pub(in crate::storage::move_kernel) fn rewrite_path_references(
    plan: &MovePlan,
) -> MoveResult<(Vec<MovePathRewrite>, Vec<MoveManualFollowup>)> {
    let old_abs = normalize_slashes(&plan.source_entity_path);
    let new_abs = normalize_slashes(&plan.destination_entity_path);

    let mut relative_pairs = Vec::new();
    if let (Ok(old_rel), Ok(new_rel)) = (
        safe_strip_prefix(&plan.source_entity_path, &plan.source_git_worktree_root),
        safe_strip_prefix(
            &plan.destination_entity_path,
            &plan.source_git_worktree_root,
        ),
    ) {
        relative_pairs.push((normalize_slashes(&old_rel), normalize_slashes(&new_rel)));
    }
    if let (Ok(old_rel), Ok(new_rel)) = (
        safe_strip_prefix(&plan.source_entity_path, &plan.target_git_worktree_root),
        safe_strip_prefix(
            &plan.destination_entity_path,
            &plan.target_git_worktree_root,
        ),
    ) {
        relative_pairs.push((normalize_slashes(&old_rel), normalize_slashes(&new_rel)));
    }

    let mut rewritten = Vec::new();
    let mut followups = Vec::new();

    for file in &plan.path_reference_files {
        let file_path = file.clone();
        if !file_path.exists() {
            followups.push(MoveManualFollowup {
                path: file_path,
                reason: "tracked reference file missing on disk".to_string(),
            });
            continue;
        }

        let bytes = fs::read(&file_path)?;
        let Ok(previous_content) = String::from_utf8(bytes) else {
            followups.push(MoveManualFollowup {
                path: file_path,
                reason: "binary or non-utf8 content requires manual rewrite".to_string(),
            });
            continue;
        };

        let mut replacements = Vec::new();
        let mut replaced = previous_content.clone();
        if replaced.contains(&old_abs) {
            replaced = replaced.replace(&old_abs, &new_abs);
            replacements.push(MoveTextReplacement {
                before: old_abs.clone(),
                after: new_abs.clone(),
            });
        }
        for (old_rel, new_rel) in &relative_pairs {
            if !old_rel.is_empty() && replaced.contains(old_rel) {
                replaced = replaced.replace(old_rel, new_rel);
                replacements.push(MoveTextReplacement {
                    before: old_rel.clone(),
                    after: new_rel.clone(),
                });
            }
        }

        if replaced == previous_content {
            followups.push(MoveManualFollowup {
                path: file_path,
                reason: "no rewrite candidate matched file content".to_string(),
            });
            continue;
        }

        let Some((repo_root, repo_relative_path)) = tracked_repo_for_file(
            &file_path,
            &plan.source_git_worktree_root,
            &plan.target_git_worktree_root,
        ) else {
            followups.push(MoveManualFollowup {
                path: file_path,
                reason: "tracked rewrite file did not resolve under source or target git root"
                    .to_string(),
            });
            continue;
        };

        fs::write(&file_path, replaced.as_bytes())?;
        rewritten.push(MovePathRewrite {
            path: file_path,
            repo_root: repo_root.to_path_buf(),
            repo_relative_path,
            replacements,
            previous_content: None,
        });
    }

    Ok((rewritten, followups))
}

pub(in crate::storage::move_kernel) fn normalize_slashes(path: &Path) -> String {
    crate::workspace::normalize_slashes(path)
}
