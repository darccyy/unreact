use std::collections::HashMap;

/// Alias of common result type
//TODO Rename enum
pub type UnreactResult<T> = Result<T, UnreactError>;

/// Custom error message for this module
//TODO Rename enum
//TODO Rename kinds
//TODO Change one-use kinds to unit
//TODO Remove unused
#[derive(Debug)]
pub enum UnreactError {
  InitFail(String),
  DirNotExist(String),

  TemplateNotExist(String),

  ScssFail(String),
  MinifyFail(String),

  RenderFail(String),
  RegisterPartialFail(String),
  RegisterInbuiltPartialFail(String),

  ServerFail(crate::server::ServerError),

  WriteFileFail(String),
  RemoveDirFail(String),
  CreateDirFail(String),
  CopyDirFail(String),
  ReadDirFail(String),
}

impl std::error::Error for UnreactError {}
impl std::fmt::Display for UnreactError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    // TODO ? Change this format ?
    write!(f, "Unreact Error:\n{:?}", self)
  }
}

/// Alias of hashmap
pub type FileMap = HashMap<String, String>;

/// File object
#[derive(Debug)]
pub struct File {
  pub path: String,
  pub content: String,
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
