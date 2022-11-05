use std::{collections::HashMap, error::Error, fs, path::Path};

use handlebars::Handlebars;
use serde_json::Value;

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

/// Config for directories and options
#[derive(Debug)]
pub struct AppConfig {
  /// Directory of output files
  pub build: String,
  /// Directory of templates and partials (`.hbs`)
  pub templates: String,
  /// Directory of static public assets, such as images
  pub public: String,
  /// Directory of styles (`.scss`)
  pub styles: String,
  /// If links between documents should include `.html` and `/index.html`
  pub _include_extension: bool,
}

impl Default for AppConfig {
  fn default() -> Self {
    AppConfig {
      build: "build".to_string(),
      templates: "templates".to_string(),
      public: "public".to_string(),
      styles: "styles".to_string(),
      _include_extension: true,
    }
  }
}

impl AppConfig {
  pub fn github_pages() -> Self {
    AppConfig {
      build: "docs".to_string(),
      _include_extension: false,
      ..Default::default()
    }
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

/// API interface object
#[derive(Debug)]
pub struct App {
  /// Config options for app
  config: AppConfig,
  /// List of templates as file hashmap
  templates: FileMap,
  /// List of styles as file hashmap
  styles: FileMap,
  /// List of pages as file list
  pages: Vec<File>,
  /// Whether app should compile in dev mode
  /// If true, localhost server will be opened
  pub is_dev: bool,
}

impl App {
  /// Create new API interface
  /// Use `Default::default()` for default config
  pub fn new(config: AppConfig, is_dev: bool) -> AppResult<Self> {
    Self::check_dirs(&config)?;

    Ok(App {
      templates: Self::load_templates(&config)?,
      styles: Self::load_styles(&config)?,
      pages: Vec::new(),
      config,
      is_dev,
    })
  }

  /// Returns as error if any value of `config` are not valid directories.
  /// Creates build directory
  fn check_dirs(config: &AppConfig) -> AppResult<()> {
    // Collate directory names
    let dirs = vec![&config.templates, &config.public, &config.styles];

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
    if Path::new(&format!("./{}", config.build)).exists() {
      fs::remove_dir_all(format!("./{}", config.build))?;
    }
    // Create new build directory
    fs::create_dir(format!("./{}", config.build))?;
    // Create generic subfolders
    fs::create_dir(format!("./{}/styles", config.build))?;
    fs::create_dir(format!("./{}/public", config.build))?;

    Ok(())
  }

  /// Load all templates in directory of `templates` property in `config`
  fn load_templates(config: &AppConfig) -> AppResult<FileMap> {
    let mut templates = FileMap::new();
    load_filemap(&mut templates, &config.templates, "")?;
    Ok(templates)
  }

  /// Import all scss files in directory of `styles` property in `config`
  fn load_styles(config: &AppConfig) -> AppResult<FileMap> {
    let mut styles = FileMap::new();
    load_filemap(&mut styles, &config.styles, "")?;
    Ok(styles)
  }

  /// Render a template with data
  pub fn render(&self, name: &str, data: &Value) -> AppResult<String> {
    // Get template string from name
    let template = match self.templates.get(name) {
      Some(s) => s,
      None => {
        return Err(Box::new(AppError(format!(
          "Could not find template '{name}'"
        ))))
      }
    };

    // Create handlebars registry
    let mut reg = Handlebars::new();

    // Register all other templates as partials
    for (k, v) in &self.templates {
      reg.register_partial(k, v)?;
    }

    // Register default partials
    //TODO Change for production mode
    reg.register_partial(
      "URL",
      format!(
        "file:///{dir}/{build}",
        dir = std::env::current_dir().unwrap().to_str().unwrap(),
        build = &self.config.build
      ),
    )?;

    // Render template
    Ok(reg.render_template(template, data)?)
  }

  /// Register new page (file) with any path
  ///TODO Move `render` function here
  pub fn page(&mut self, path: &str, content: &str) -> AppResult<&mut Self> {
    self.pages.push(File::new(path, content));

    Ok(self)
  }

  /// Register index page (`./index.html`)
  pub fn index(&mut self, content: &str) -> AppResult<&mut Self> {
    self.page("index", content)
  }

  /// Register 404 not found page (`./404.html`)
  pub fn not_found(&mut self, content: &str) -> AppResult<&mut Self> {
    self.page("404", content)
  }

  /// Create all files in production mode
  pub fn finish(&mut self) -> AppResult<&mut Self> {
    // Create pages
    for file in &self.pages {
      let parent = &self.config.build;
      // Create folder recursively
      create_dir_all_safe(parent, &file.path)?;
      // Create file
      fs::write(format!("./{parent}/{}.html", file.path), &file.content)?;
    }

    // Create styles
    for (path, content) in &self.styles {
      let parent = format!("{}/{}", self.config.build, self.config.styles);
      // Create folder recursively
      create_dir_all_safe(&parent, &path)?;
      // Create file - Convert from `scss` to `css` with `grass`
      fs::write(
        format!("./{parent}/{path}.css"),
        grass::from_string(content.to_string(), &grass::Options::default())?,
      )?;
    }

    // Copy public files
    for file in fs::read_dir(format!("./{}", &self.config.public))?.flatten() {
      if let Some(name) = file.file_name().to_str() {
        fs::copy(
          file.path(),
          format!("./{}/public/{}", self.config.build, name),
        )?;
      }
    }

    if self.is_dev {
      Self::listen();
    }

    Ok(self)
  }

  /// Open local server and listen
  fn listen() {
    //TODO Listen here
  }
}

/// Recursively read files from tree directory.
/// `templates`: Mutable borrow to hashmap
/// `parent`: Directory to collate all templates
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
