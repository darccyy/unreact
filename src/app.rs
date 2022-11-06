use std::{fs, path::Path};

use handlebars::Handlebars;
use serde_json::Value;

use crate::{
  create_dir_all_safe, load_filemap, merge_json, server, AppError, AppResult, File, FileMap,
  DEV_BUILD_DIR,
};

/// Config for directories and options
///
/// Use `AppConfig::default()` for default config
///
/// Use `AppConfig::github_pages()` for recommended default for hosting on GitHub Pages (Builds to `./docs`)
#[derive(Debug)]
pub struct AppConfig {
  /// Directory of output files - build directory
  ///
  /// For production. Temporary folder `./.devbuild` is used in development
  ///
  /// Default: `"build"`, or `"docs"` with `AppConfig::github_pages()`
  pub build: String,
  /// Directory of templates and partials (`.hbs`)
  ///
  /// Can contain nested files
  ///
  /// Default: `"templates"`
  pub templates: String,
  /// Directory of static public assets, such as images
  ///
  /// Can contain nested files
  ///
  /// Default: `"public"`
  pub public: String,
  /// Directory of styles (`.scss`)
  ///
  /// Can contain nested files
  ///
  /// Default: `"styles"`
  pub styles: String,
  /// If warning is sent in dev mode
  ///
  /// Default: `true`
  pub dev_warning: bool,
  /// If `html` and `css` files are minified in build
  ///
  /// Default: `true`
  pub minify: bool,
}

impl Default for AppConfig {
  fn default() -> Self {
    AppConfig {
      build: "build".to_string(),
      templates: "templates".to_string(),
      public: "public".to_string(),
      styles: "styles".to_string(),
      dev_warning: true,
      minify: true,
    }
  }
}

impl AppConfig {
  /// Returns recommended default for hosting on GitHub Pages (Builds to `./docs`)
  pub fn github_pages() -> Self {
    AppConfig {
      build: "docs".to_string(),
      ..Default::default()
    }
  }
}

/// API interface object
///
/// Create with `App::new()`
#[derive(Debug)]
pub struct App {
  /// Config options for app, see `AppConfig`
  config: AppConfig,
  /// List of templates as file hashmap
  templates: FileMap,
  /// List of styles as file hashmap
  styles: FileMap,
  /// List of pages as file list
  pages: Vec<File>,
  /// Whether app should compile in dev mode
  ///
  /// If true, localhost server will be created
  is_dev: bool,
  /// URL of production server
  url: String,
  /// Global variables
  globals: Value,
}

impl App {
  /// Create new API interface
  ///
  /// Use `AppConfig::default()` as `config` for default config
  ///
  /// Use `AppConfig::github_pages()` as `config` for recommended config for hosting on GitHub Pages default (Builds to `./docs`)
  pub fn new(config: AppConfig, is_dev: bool, url: &str) -> AppResult<Self> {
    // Convert build directory to constant dev build directory if is dev
    let config = if is_dev {
      {
        AppConfig {
          build: DEV_BUILD_DIR.to_string(),
          ..config
        }
      }
    } else {
      config
    };

    // Check that directories exists
    Self::check_dirs(&config)?;

    // Create interface
    Ok(App {
      templates: Self::load_templates(&config)?,
      styles: Self::load_styles(&config)?,
      pages: Vec::new(),
      config,
      is_dev,
      url: url.to_string(),
      globals: Value::Null,
    })
  }

  /// Returns as error if any value of `config` are not valid directories
  ///
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

    // Create new build directory and generic subfolders
    let dirs = vec!["", "/styles", "/public"];
    for dir in dirs {
      fs::create_dir(format!("./{}{}", config.build, dir))?;
    }

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

  /// Set global variables to new `serde_json::Value`
  // ? Create getter ?
  pub fn set_globals(&mut self, data: Value) -> AppResult<&mut Self> {
    self.globals = data;
    Ok(self)
  }

  /// Render a template with data
  // ? Make public ?
  fn render(&self, name: &str, data: &Value) -> AppResult<String> {
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

    // Register inbuilt partials
    // Base url for site
    reg.register_partial(
      "URL",
      if self.is_dev {
        format!("http://{}", server::ADDRESS)
      } else {
        self.url.to_string()
      },
    )?;
    // Script for development
    // Is not registered if `dev_warning` in config is false
    if self.is_dev && self.config.dev_warning {
      reg.register_partial("DEV_SCRIPT", server::DEV_SCRIPT)?;
    }
    // Simple link
    reg.register_partial(
      "LINK",
      r#"
        <a href="{{>URL}}/{{to}}"> {{>@partial-block}} </a>
      "#,
    )?;
    // Simple style tag
    reg.register_partial(
      "STYLE",
      r#"
        <link rel="stylesheet" href="{{>URL}}/styles/{{name}}.css" />
      "#,
    )?;

    // ? Remove `.clone` (2x) ? how ?
    let mut data = data.clone();
    if !self.globals.is_null() {
      merge_json(&mut data, self.globals.clone());
    }

    // Render template
    Ok(reg.render_template(template, &data)?)
  }

  /// Register new page (file) with any path, with template
  ///
  /// `path`: Output path in build directory, **without** `.html` extension
  ///
  /// `template`: Name of template to render
  ///
  /// `data`: JSON data to render with (use `serde_json::json!` macro)
  pub fn page(&mut self, path: &str, template: &str, data: &Value) -> AppResult<&mut Self> {
    self
      .pages
      .push(File::new(path, &self.render(template, data)?));
    Ok(self)
  }

  /// Register new page (file) with any path, without template (plain)
  ///
  /// `path`: Output path in build directory, **without** `.html` extension
  ///
  /// `content`: Raw text content to write to file, without template
  pub fn page_plain(&mut self, path: &str, content: &str) -> AppResult<&mut Self> {
    self.pages.push(File::new(path, content));
    Ok(self)
  }

  /// Register index page (`./index.html`)
  ///
  /// Alias of `app.page("index", ...)`
  pub fn index(&mut self, template: &str, data: &Value) -> AppResult<&mut Self> {
    self.page("index", template, data)
  }

  /// Register 404 (not found) page (`./404.html`)
  ///
  /// Alias of `app.page("404", ...)`
  pub fn not_found(&mut self, template: &str, data: &Value) -> AppResult<&mut Self> {
    self.page("404", template, data)
  }

  /// Create all files in production mode
  pub fn finish(&mut self) -> AppResult<&mut Self> {
    // Create pages
    for file in &self.pages {
      let parent = &self.config.build;
      // Create folder recursively
      create_dir_all_safe(parent, &file.path)?;

      // Minify if enabled
      let output = if self.config.minify {
        // Minified html
        use minify_html::{minify, Cfg};
        String::from_utf8_lossy(&minify(
          &file.content.as_bytes(),
          &Cfg {
            do_not_minify_doctype: true,
            keep_comments: true,
            ..Cfg::default()
          },
        ))
        .to_string()
      } else {
        // Un-minified file
        file.content.to_string()
      };

      // Create file
      fs::write(format!("./{parent}/{}.html", file.path), &output)?;
    }

    // Create styles
    for (path, content) in &self.styles {
      let parent = format!("{}/{}", self.config.build, self.config.styles);
      // Create folder recursively
      create_dir_all_safe(&parent, &path)?;

      // Convert from scss to css
      let parsed = grass::from_string(content.to_string(), &grass::Options::default())?;

      // Minify if enabled
      let output = if self.config.minify {
        // Minified css
        use css_minify::optimizations::{Level, Minifier};
        Minifier::default()
          .minify(&parsed, Level::Two)
          .expect(&format!("Could not minify css in file '{path}'"))
      } else {
        // Un-minified file
        parsed
      };

      // Create file - Convert from `scss` to `css` with `grass`
      fs::write(format!("./{parent}/{path}.css"), output)?;
    }

    // Copy public files
    dircpy::copy_dir(
      format!("./{}", &self.config.public),
      format!("./{}/public", self.config.build),
    )?;

    // Open local server if in dev mode
    if self.is_dev {
      Self::listen();
    }

    Ok(self)
  }

  /// Open local server and listen
  fn listen() {
    server::listen();
  }
}
