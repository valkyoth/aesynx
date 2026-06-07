use std::path::PathBuf;

pub fn root() -> Result<PathBuf, &'static str> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let Some(tools_dir) = manifest_dir.parent() else {
        return Err("cannot resolve tools directory from CARGO_MANIFEST_DIR");
    };
    let Some(root) = tools_dir.parent() else {
        return Err("cannot resolve workspace root from tools directory");
    };

    Ok(root.to_path_buf())
}
