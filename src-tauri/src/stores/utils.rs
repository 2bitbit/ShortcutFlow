use anyhow::Result;
use std::path::{Path, PathBuf};

pub fn get_all_json_file_paths(dir_path: Vec<&Path>) -> Result<Vec<PathBuf>> {
    let mut all_json_files: Vec<PathBuf> = Vec::new();
    for dir in dir_path {
        let json_files: Vec<PathBuf> = walkdir::WalkDir::new(dir)
            .into_iter()
            .map(|item| item.map_err(|e| anyhow::anyhow!(e)))
            .collect::<Result<Vec<walkdir::DirEntry>>>()? // 每一个json都可读，才进行后续逻辑。
            .into_iter()
            .filter_map(|e| {
                if e.file_type().is_file()
                    && e.path().extension().and_then(|s| s.to_str()) == Some("json")
                {
                    Some(e.path().to_path_buf())
                } else {
                    None
                }
            })
            .collect();
        all_json_files.extend(json_files);
    }
    Ok(all_json_files)
}
