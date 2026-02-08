use std::{fs, path::Path};

pub fn walk_files<F>(dir: &str, extension: &str, mut callback: F)
where
    F: FnMut(&Path, String),
{
    fn walk_recursive<F>(dir: &Path, extension: &str, callback: &mut F)
    where
        F: FnMut(&Path, String),
    {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk_recursive(&path, extension, callback);
                } else if path.extension().and_then(|s| s.to_str()) == Some(extension)
                    && let Ok(content) = fs::read_to_string(&path)
                {
                    callback(&path, content);
                }
            }
        }
    }

    walk_recursive(Path::new(dir), extension, &mut callback);
}
