use handlebars::{RenderError, TemplateError};
use std::collections::HashMap;

/// Alias of result type, with [UnreactError]
//TODO Rename enum
pub type UnreactResult<T> = Result<T, UnreactError>;

/// Custom error message for Unreact
///
/// See enum variants for detailed description of each
//TODO Rename enum
#[derive(Debug)]
pub enum UnreactError {
  /// Given directory does not exist
  ///
  /// Try:
  ///  - Verifying config directories exist in workspace
  DirNotExist(String),

  /// Cannot find template with name given
  ///
  /// Try:
  ///  - Removing file extension `.hbs` from template name
  ///  - Verifying template name matches path in template directory
  TemplateNotExist(String),

  /// Failed to convert `.scss` to `.css`
  ///
  /// Try:
  ///  - Checking for any bugs or unsupported features in the `.scss` file
  ///
  /// See: [grass](https://crates.io/crates/grass) crate
  ScssConvertFail(String, String),

  /// Failed to minify `.css` file
  ///
  /// Try:
  ///  - Checking for any bugs or unsupported features in the original `.css` or `.scss` file
  ///
  /// See: [css-minify](https://crates.io/crates/css-minify) crate
  MinifyCssFail(String, String),

  /// Failed to render template
  ///
  /// Try:
  ///  - Checking for any bugs or unsupported features in the `.hbs` file
  ///
  /// See: [handlebars](https://crates.io/crates/handlebars) crate
  HandlebarsFail(String, RenderError),

  /// Failed to register partial
  ///
  /// All `.hbs` templates are automatically registered as partials
  ///
  /// Try:
  ///  - Checking for any bugs or unsupported features in the `.hbs` file
  ///
  /// See: [handlebars](https://crates.io/crates/handlebars) crate
  RegisterPartialFail(String, TemplateError),

  /// Failed to register inbuilt partial
  ///
  /// Try:
  ///  - Reporting this bug [here](https://github.com/darccyy/unreact/issues/new)
  RegisterInbuiltPartialFail(String, TemplateError),

  /// An IO or FS error occurred
  IoError(std::io::Error, String),
}

impl std::error::Error for UnreactError {}
impl std::fmt::Display for UnreactError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      UnreactError::DirNotExist(path) => write!(
        f,
        "Directory does not exist at '{path}' (UnreactError::DirNotExist)"
      ),
      UnreactError::TemplateNotExist(name) => write!(
        f,
        "Template does not exist with name '{name}' (UnreactError::TemplateNotExist)"
      ),
      UnreactError::ScssConvertFail(name, err) => write!(
        f,
        "Failed to convert SCSS to CSS for '{name}' (UnreactError::ScssConvertFail) - {err:?}"
      ),
      UnreactError::MinifyCssFail(name, err) => write!(
        f,
        "Failed to minify CSS file for '{name}' (UnreactError::MinifyCssFail) - {err:?}"
      ),
      UnreactError::HandlebarsFail(name, err) => write!(
        f,
        "Failed to render template with name '{name}' (UnreactError::RenderFail) - {err:?}"
      ),
      UnreactError::RegisterPartialFail(name, err) => write!(
        f,
        "Failed to register custom partial with name '{name}' (UnreactError::RegisterPartialFail) - {err:?}"
      ),
      UnreactError::RegisterInbuiltPartialFail(name, err) => write!(
        f,
        "Failed to register *inbuilt* partial '{name}' (UnreactError::RegisterInbuiltPartialFail) - {err:?}"
      ),
      UnreactError::IoError(err, path) => write!(
        f,
        "File Error: {err:?}, at path '{path}' (UnreactError::IoError)"
      ),
    }
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
