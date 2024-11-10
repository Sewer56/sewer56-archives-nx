use crate::api::filedata::FromFilePathProvider;
use crate::api::packing::packer_file::PackerFile;
use crate::api::traits::*;
use alloc::sync::Arc;
use std::fs::*;
use std::path::*;

// TODO: Optimized version of this struct that doesn't use `std::fs`.
//       for now I'm not concerned because binary size for packing is not as big a priority as for
//       unpacking.

trait PathExt {
    fn normalize_separators(&self) -> String;
}

impl PathExt for Path {
    fn normalize_separators(&self) -> String {
        let path_str = self.to_string_lossy();
        #[cfg(windows)]
        {
            path_str.replace('\\', "/")
        }
        #[cfg(not(windows))]
        {
            path_str.into_owned()
        }
    }
}

/// Iterates through all packable files from within a given directory,
/// passing each found file to the provided callback function.
///
/// # Arguments
///
/// * `directory_path` - The full path to the directory to search
/// * `callback` - Function that will be called for each file found
///
/// # Errors
///
/// Returns an error if there are issues accessing the directory or files.
pub fn find_files<P, F>(directory_path: P, mut callback: F) -> Result<(), FileProviderError>
where
    P: AsRef<Path>,
    F: FnMut(PackerFile),
{
    walk_directory(
        directory_path.as_ref(),
        directory_path.as_ref(),
        &mut callback,
    )
}

fn walk_directory<F>(
    current_path: &Path,
    base_path: &Path,
    callback: &mut F,
) -> Result<(), FileProviderError>
where
    F: FnMut(PackerFile),
{
    for entry in read_dir(current_path)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            walk_directory(&path, base_path, callback)?;
        } else if file_type.is_file() {
            let metadata = entry.metadata()?;
            if let Ok(relative_path) = path.strip_prefix(base_path) {
                let relative_path_str = relative_path.normalize_separators();

                let provider = Arc::new(FromFilePathProvider::new(path.to_str().unwrap())?);
                let packer_file = PackerFile::new(relative_path_str, metadata.len(), provider);

                callback(packer_file);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn finds_files_in_directory() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();
        create_dir(base_path.join("subdir")).unwrap();

        let mut file1 = File::create(base_path.join("file1.txt")).unwrap();
        write!(file1, "test1").unwrap();

        let mut file2 = File::create(base_path.join("subdir/file2.txt")).unwrap();
        write!(file2, "test2").unwrap();

        let mut files = Vec::new();
        find_files(base_path, |file| files.push(file)).unwrap();
        assert_eq!(files.len(), 2);

        let file_paths: Vec<_> = files.iter().map(|f| f.relative_path()).collect();
        assert!(file_paths.contains(&"file1.txt"));
        assert!(file_paths.contains(&"subdir/file2.txt"));
    }

    #[test]
    fn handles_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let mut count = 0;
        find_files(temp_dir.path(), |_| count += 1).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn correct_file_sizes() {
        let temp_dir = TempDir::new().unwrap();
        let test_path = temp_dir.path().join("test.txt");

        let mut file = File::create(&test_path).unwrap();
        write!(file, "Hello World!").unwrap();
        file.flush().unwrap();

        let mut files = Vec::new();
        find_files(temp_dir.path(), |file| files.push(file)).unwrap();

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].file_size(), 12);
    }

    #[test]
    fn path_separators_handling() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        create_dir_all(base_path.join("dir1/dir2")).unwrap();
        let mut file = File::create(base_path.join("dir1/dir2/file.txt")).unwrap();
        write!(file, "test").unwrap();

        let mut files = Vec::new();
        find_files(base_path, |file| files.push(file)).unwrap();

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].relative_path(), "dir1/dir2/file.txt");
    }

    #[test]
    fn handles_invalid_directory() {
        let result = find_files("nonexistent_directory", |_| {});
        assert!(result.is_err());
    }
}
