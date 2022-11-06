mod app;
mod server;

use std::{collections::HashMap, error::Error, fs, path::Path};

pub use app::{App, AppConfig};

pub const DEV_BUILD_DIR: &str = ".devbuild";

/// Alias of hashmap
type FileMap = HashMap<String, String>;

/// Alias of common result type
pub type AppResult<T> = Result<T, Box<dyn std::error::Error>>;

/// Custom error message for this module
///TODO Make better !!!
#[derive(Debug)]
pub struct AppError(String);

impl Error for AppError {}
impl std::fmt::Display for AppError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "ERROR <{}>", self.0)
  }
}

/// File object
#[derive(Debug)]
pub struct File {
  path: String,
  content: String,
}

impl File {
  /// Create new `File` struct
  pub fn new(path: &str, content: &str) -> Self {
    File {
      path: path.to_string(),
      content: content.to_string(),
    }
  }
}

/// Recursively read files from tree directory
///
/// `templates`: Mutable borrow to hashmap
///
/// `parent`: Directory to collate all templates
///
/// `child`: Path of subdirectories (not including `parent`)
fn load_filemap(map: &mut FileMap, parent: &str, child: &str) -> AppResult<()> {
  // Full path, relative to workspace, of directory
  let dir_path = format!("./{parent}/{child}");

  // Loop files in directory
  for file in fs::read_dir(dir_path)?.flatten() {
    if let Some(path) = file.path().to_str() {
      let path = path.replace("\\", "/");
      if let Some(name) = file.file_name().to_str() {
        // Only include first slash if child directory is not empty
        let slash = if child.is_empty() { "" } else { "/" };

        // If is folder
        if Path::new(&path).is_dir() {
          // Recurse function
          load_filemap(map, parent, &format!("{child}{slash}{name}",))?;
        } else {
          // Add to templates
          let content = fs::read_to_string(file.path())?;
          // Get file name without extension
          if let Some(file_name) = get_file_name(&file) {
            map.insert(format!("{child}{slash}{file_name}",), content);
          }
        }
      }
    }
  }

  Ok(())
}

/// Create folder recursively
fn create_dir_all_safe(parent: &str, child: &str) -> AppResult<()> {
  let folders = child.split("/").collect::<Vec<_>>();
  for i in 1..folders.len() {
    let path = format!("./{}/{}", parent, folders.get(0..i).unwrap().join("/"));
    // Check if exists, create if not
    if !Path::new(&path).exists() {
      fs::create_dir(path)?;
    }
  }

  Ok(())
}

/// Convert `DirEntry` to string and get file name without extension
fn get_file_name(path: &fs::DirEntry) -> Option<String> {
  Some(
    path
      .path()
      .to_str()?
      .replace('\\', "/")
      .split('/')
      .last()?
      .split('.')
      .next()?
      .to_owned(),
  )
}
