use super::*;

pub(super) fn serialize_normalized_path<S>(path: &PathBuf, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&normalize_slashes(path))
}

pub(super) fn deserialize_pathbuf<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(PathBuf::from(String::deserialize(deserializer)?))
}

pub(super) fn serialize_normalized_path_vec<S>(
    paths: &[PathBuf],
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let normalized: Vec<String> = paths.iter().map(|path| normalize_slashes(path)).collect();
    normalized.serialize(serializer)
}

pub(super) fn deserialize_pathbuf_vec<'de, D>(deserializer: D) -> Result<Vec<PathBuf>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Vec::<String>::deserialize(deserializer)?
        .into_iter()
        .map(PathBuf::from)
        .collect())
}

pub(in crate::storage::move_kernel) fn path_buf_is_empty(path: &PathBuf) -> bool {
    path.as_os_str().is_empty()
}
