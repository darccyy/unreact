mod app;
mod server;
mod types;

use std::{fs, path::Path};

use serde_json::Value;

pub use app::{Config, Unreact};
pub use types::{UnreactError, UnreactResult, FileMap, File};

/// Directory of temporary development build
pub const DEV_BUILD_DIR: &str = ".devbuild";

/// Recursively read files from tree directory
///
/// `templates`: Mutable borrow to hashmap
///
/// `parent`: Directory to collate all templates
///
/// `child`: Path of subdirectories (not including `parent`)
// ? Change to `std::io::Result` ?
fn load_filemap(map: &mut FileMap, parent: &str, child: &str) -> UnreactResult<()> {
  // Full path, relative to workspace, of directory
  let dir_path = format!("./{parent}/{child}");

  // Read directory
  // ? Remove clone ?
  let dir_path_clone = dir_path.clone();
  let dir = match fs::read_dir(dir_path) {
    Ok(x) => x,
    Err(_) => return Err(UnreactError::ReadDirFail(dir_path_clone)),
  };

  // Loop files in directory
  for file in dir.flatten() {
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
          let content = match fs::read_to_string(file.path()) {
            Ok(x) => x,
            Err(_) => {
              return Err(UnreactError::ReadDirFail(
                file
                  .path()
                  .to_str()
                  //TODO Handle
                  .expect("Could not convert path to string")
                  .to_string(),
              ));
            }
          };

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
fn create_dir_all_safe(parent: &str, child: &str) -> UnreactResult<()> {
  let folders = child.split("/").collect::<Vec<_>>();
  for i in 1..folders.len() {
    let path = format!("./{}/{}", parent, folders.get(0..i).unwrap().join("/"));
    // Check if exists, create if not
    if !Path::new(&path).exists() {
      if let Err(_) = fs::create_dir(path) {
        return Err(UnreactError::CreateDirFail(format!(
          "./{}/{}",
          parent,
          folders.get(0..i).unwrap().join("/")
        )));
      }
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

/// Merge one `serde_json` value with another
fn merge_json(a: &mut Value, b: Value) {
  if let Value::Object(a) = a {
    if let Value::Object(b) = b {
      for (k, v) in b {
        if v.is_null() {
          a.remove(&k);
        } else {
          merge_json(a.entry(k).or_insert(Value::Null), v);
        }
      }

      return;
    }
  }

  *a = b;
}
