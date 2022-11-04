use std::{collections::HashMap, error::Error, fs, path::Path};

// use handlebars::template;
use serde_json::Value;

/// Alias of hashmap
type Templates = HashMap<String, String>;

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

/// Options for directories
#[derive(Debug)]
pub struct AppOptions {
  /// Directory of output files
  build: String,
  /// Directory of templates and partials (`.hbs`)
  templates: String,
  /// Directory of static public assets, such as images
  public: String,
  /// Directory of styles (`.scss`)
  styles: String,
  /// If links between documents should include `.html` and `/index.html`
  _include_extension: bool,
}

impl Default for AppOptions {
  fn default() -> Self {
    Self {
      build: "build".to_string(),
      templates: "templates".to_string(),
      public: "public".to_string(),
      styles: "styles".to_string(),
      _include_extension: true,
    }
  }
}

/// API interface object
#[derive(Debug)]
pub struct App {
  pub options: AppOptions,
  pub templates: Templates,
}

impl App {
  /// Create new API interface
  /// Use `Default::default()` for default options
  pub fn new(options: AppOptions) -> AppResult<Self> {
    Self::check_dirs(&options)?;

    Ok(App {
      templates: Self::load_templates(&options)?,
      options,
    })
  }

  /// Returns as error if any value of `options` are not valid directories.
  /// Creates build directory
  fn check_dirs(options: &AppOptions) -> AppResult<()> {
    // Collate directory names
    let dirs = vec![&options.templates, &options.public, &options.styles];

    // Loop directories that should exist
    for dir in dirs {
      // Check if directory exists
      let path = Path::new(dir);
      if !path.is_dir() {
        return Err(Box::new(AppError(format!(
          "Directory `{dir}` does not exist"
        ))));
      }
    }

    // Remove build directory if exists
    if Path::new(&format!("./{}", options.build)).exists() {
      fs::remove_dir_all(format!("./{}", options.build))?;
    }
    // Create new build directory
    fs::create_dir(format!("./{}", options.build))?;

    Ok(())
  }

  /// Load all templates in directory of `templates` property in `options`
  fn load_templates(options: &AppOptions) -> AppResult<Templates> {
    let mut templates = Templates::new();
    App::load_templates_from(&mut templates, &options.templates, "")?;
    Ok(templates)
  }

  /// Recursively read templates from tree directory.
  /// `templates`: Mutable borrow to hashmap
  /// `parent`: Directory to collate all templates
  /// `child`: Path of subdirectories (not including `parent`)
  fn load_templates_from(templates: &mut Templates, parent: &str, child: &str) -> AppResult<()> {
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
            App::load_templates_from(templates, parent, &format!("{child}{slash}{name}",))?;
          } else {
            // Add to templates
            let contents = fs::read_to_string(file.path())?;
            // Get file name without extension
            if let Some(file_name) = get_file_name(&file) {
              templates.insert(format!("{child}{slash}{file_name}",), contents);
            }
          }
        }
      }
    }

    Ok(())
  }

  pub fn render(template: &str, data: &Value) -> AppResult<String> {
    todo!();
  }

  pub fn page(&mut self, path: &str, content: &str) -> AppResult<&mut Self> {
    todo!();
  }

  pub fn index(&mut self, content: &str) -> AppResult<&mut Self> {
    todo!();
  }

  pub fn not_found(&mut self, content: &str) -> AppResult<&mut Self> {
    todo!();
  }

  pub fn finish(&mut self) -> AppResult<&mut Self> {
    todo!();
  }

  pub fn listen(&mut self) -> AppResult<&mut Self> {
    todo!();
  }
}

/// Convert `DirEntry` to string and get file name without extension
pub fn get_file_name(path: &fs::DirEntry) -> Option<String> {
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
