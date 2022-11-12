use handlebars::Handlebars;
use serde_json::Value;
use std::{fs, path::Path};

use crate::{
  create_dir_all_safe, load_filemap, merge_json, server, File, FileMap, UnreactError,
  UnreactResult, DEV_BUILD_DIR,
};

/// Config for directories and options
///
/// Use `Config::default()` for default config
#[derive(Debug)]
pub struct Config {
  /// Directory of output files - build directory
  ///
  /// For production. Temporary folder `./.devbuild` is used in development
  ///
  /// Default: `"build"`
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

impl Default for Config {
  fn default() -> Self {
    Config {
      build: "build".to_string(),
      templates: "templates".to_string(),
      public: "public".to_string(),
      styles: "styles".to_string(),
      dev_warning: true,
      minify: true,
    }
  }
}

/// API interface object
///
/// Create with `Unreact::new()`
#[derive(Debug)]
pub struct Unreact {
  /// Config options for app, see `Config`
  config: Config,
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

impl Unreact {
  /// Create new API interface
  ///
  /// Use `Config::default()` as `config` for default config
  ///
  /// # Examples
  ///
  /// Compiles a basic site
  ///
  /// ```
  /// use unreact::prelude::*;
  ///
  /// fn main() -> UnreactResult<()> {
  ///   let mut app = Unreact::new(Config::default(), false, "https://mysite.com")?;
  ///
  ///   app.page_plain("index", "This is my site")
  ///     .finish()?;
  ///
  ///   Ok(())
  /// }
  /// ```
  pub fn new(config: Config, is_dev: bool, url: &str) -> UnreactResult<Self> {
    // Convert build directory to constant dev build directory if is dev
    let config = if is_dev {
      {
        Config {
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
    Ok(Unreact {
      templates: Self::load_templates(&config)?,
      styles: Self::load_styles(&config)?,
      pages: Vec::new(),
      config,
      is_dev,
      url: url.to_string(),
      globals: Value::Null,
    })
  }

  /// Set global variables to new `serde_json::Value`
  ///
  /// # Examples
  ///
  /// Creates a global variable
  ///
  /// ```
  /// use unreact::prelude::*;
  /// use serde_json::json;
  ///
  /// fn main() -> UnreactResult<()> {
  ///   let mut app = Unreact::new(Config::default(), false, "https://mysite.com")?;
  ///
  ///   app.set_globals(json!({"my_global": "From global! :)"}));
  ///
  ///   Ok(())
  /// }
  /// ```
  // ? Create getter ?
  pub fn set_globals(&mut self, data: Value) -> &mut Self {
    self.globals = data;
    self
  }

  /// Register new page (file) with any path, without template (plain)
  ///
  /// `path`: Output path in build directory, **without** `.html` extension
  ///
  /// `content`: Raw text content to write to file, without template
  ///
  /// # Examples
  ///
  /// Renders two files with raw text
  ///
  /// ```
  /// use unreact::prelude::*;
  ///
  /// fn main() -> UnreactResult<()> {
  ///   let mut app = Unreact::new(Config::default(), false, "https://mysite.com")?;
  ///
  ///   // Renders to `./build/index.html`
  ///   app.page_plain("index", "This is my site");
  ///   // Renders to `./build/path/file.html`
  ///   app.page_plain("path/file", "This file is in ./build/path/file.html");
  ///
  ///   app.finish()?;
  ///   Ok(())
  /// }
  /// ```
  pub fn page_plain(&mut self, path: &str, content: &str) -> &mut Self {
    self.pages.push(File::new(path, content));
    self
  }

  /// Register new page (file) with any path, with template
  ///
  /// `path`: Output path in build directory, **without** `.html` extension
  ///
  /// `template`: Name of template to render, **without** `.hbs` extension
  ///
  /// `data`: JSON data to render with (use `serde_json::json!` macro)
  ///
  /// # Examples
  ///
  /// Renders two files with templates
  ///
  /// ```
  /// use unreact::prelude::*;
  /// use serde_json::{json, Value};
  ///
  /// fn main() -> UnreactResult<()> {
  ///   let mut app = Unreact::new(Config::default(), false, "https://mysite.com")?;
  ///
  ///   // Renders to `./build/help.html`, using `./templates/help_template.hbs`, with no data
  ///   app.page("help", "help_template", Value::Null);
  ///
  ///   // Renders to `./build/path/file.html`, using `./templates/other/template.hbs`, with a custom message
  ///   app.page("path/file", "other/template", &json!({"msg": "Hello!"}));
  ///
  ///   app.finish()?;
  ///   Ok(())
  /// }
  /// ```
  pub fn page(&mut self, path: &str, template: &str, data: &Value) -> UnreactResult<&mut Self> {
    self.page_plain(path, &self.render(template, data)?);
    Ok(self)
  }

  /// Register index page (`./index.html`), with template
  ///
  /// Alias of `app.page("index", ...)`
  ///
  /// `path`: Output path in build directory, **without** `.html` extension
  ///
  /// `template`: Name of template to render, **without** `.hbs` extension
  ///
  /// `data`: JSON data to render with (use `serde_json::json!` macro)
  ///
  /// # Examples
  ///
  /// Renders an index page with a custom message
  ///
  /// ```
  /// use unreact::prelude::*;
  /// use serde_json::{json};
  ///
  /// fn main() -> UnreactResult<()> {
  ///   let mut app = Unreact::new(Config::default(), false, "https://mysite.com")?;
  ///
  ///   // Renders to `./build/index.html`, using `./templates/standard.hbs`, with a custom message
  ///   app.index("standard", &json!({"msg": "Hello!"}));
  ///
  ///   app.finish()?;
  ///   Ok(())
  /// }
  /// ```
  pub fn index(&mut self, template: &str, data: &Value) -> UnreactResult<&mut Self> {
    self.page("index", template, data)
  }

  /// Register 404 (not found) page (`./404.html`)
  ///
  /// Alias of `app.page("404", ...)`
  ///
  /// `path`: Output path in build directory, **without** `.html` extension
  ///
  /// `template`: Name of template to render, **without** `.hbs` extension
  ///
  /// `data`: JSON data to render with (use `serde_json::json!` macro)
  ///
  /// # Examples
  ///
  /// Renders a 404 page
  ///
  /// ```
  /// use unreact::prelude::*;
  /// use serde_json::{Value};
  ///
  /// fn main() -> UnreactResult<()> {
  ///   let mut app = Unreact::new(Config::default(), false, "https://mysite.com")?;
  ///
  ///   // Renders to `./build/404.html`, using `./templates/errors/not_found.hbs`, with no data
  ///   app.not_found("errors/not_found", Value::Null);
  ///
  ///   app.finish()?;
  ///   Ok(())
  /// }
  /// ```
  pub fn not_found(&mut self, template: &str, data: &Value) -> UnreactResult<&mut Self> {
    self.page("404", template, data)
  }

  /// Create all files in production mode
  ///
  /// # Examples
  ///
  /// Compiles to `./build`, in production mode
  ///
  /// ```
  /// use unreact::prelude::*;
  ///
  /// fn main() -> UnreactResult<()> {
  ///   // Note that argument for `is_dev` is `false`
  ///   let mut app = Unreact::new(Config::default(), false, "https://mysite.com")?;
  ///
  ///   app.page_plain("index", "This is my site, in production")
  ///     .finish()?;
  ///   Ok(())
  /// }
  /// ```
  ///
  /// Compiles to `./.devbuild`, in development mode, and host to `http://127.0.0.1:8080`
  ///
  /// ```
  /// use unreact::prelude::*;
  ///
  /// fn main() -> UnreactResult<()> {
  ///   // Note that argument for `is_dev` is `true`
  ///   let mut app = Unreact::new(Config::default(), true, "https://mysite.com")?;
  ///
  ///   app.page_plain("index", "This is my site, in development")
  ///     .finish()?;
  ///   Ok(())
  /// }
  /// ```
  pub fn finish(&mut self) -> UnreactResult<&mut Self> {
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
      if let Err(err) = fs::write(format!("./{parent}/{}.html", file.path), &output) {
        return Err(UnreactError::IoError(
          err,
          format!("./{parent}/{}.html", file.path),
        ));
      }
    }

    // Create styles
    for (path, content) in &self.styles {
      let parent = format!("{}/{}", self.config.build, self.config.styles);
      // Create folder recursively
      create_dir_all_safe(&parent, &path)?;

      // Convert from scss to css
      let parsed = match grass::from_string(content.to_string(), &grass::Options::default()) {
        Ok(x) => x,
        Err(err) => {
          return Err(UnreactError::ScssConvertFail(
            path.to_string(),
            err.to_string(),
          ))
        }
      };

      // Minify if enabled
      let output = if self.config.minify {
        // Minified css
        use css_minify::optimizations::{Level, Minifier};

        match Minifier::default().minify(&parsed, Level::Two) {
          Ok(x) => x,
          Err(err) => {
            return Err(UnreactError::MinifyCssFail(
              path.to_string(),
              err.to_string(),
            ))
          }
        }
      } else {
        // Un-minified file
        parsed
      };

      // Create file - Convert from `scss` to `css` with `grass`
      if let Err(err) = fs::write(format!("./{parent}/{path}.css"), output) {
        return Err(UnreactError::IoError(err, format!("./{parent}/{path}.css")));
      }
    }

    // Copy public files
    if let Err(err) = dircpy::copy_dir(
      format!("./{}", &self.config.public),
      format!("./{}/public", self.config.build),
    ) {
      return Err(UnreactError::IoError(
        err,
        format!("./{}", &self.config.public),
      ));
    };

    // Open local server if in dev mode
    if self.is_dev {
      Self::listen();
    }

    Ok(self)
  }

  /// Render a template with data
  ///
  /// `template`: Name of template to render, **without** `.hbs` extension
  ///
  /// `data`: JSON data to render with (use `serde_json::json!` macro)
  ///
  /// # Examples
  ///
  /// Prints a template to standard output, completed with a custom message
  ///
  /// ```
  /// use unreact::prelude::*;
  ///
  /// fn main() -> UnreactResult<()> {
  ///   let mut app = Unreact::new(Config::default(), false, "https://mysite.com");
  ///
  ///   println!("{}", app.render("index", &json!({"msg": "Hello!"})));  
  ///
  ///   Ok(())
  /// }
  /// ```
  pub fn render(&self, name: &str, data: &Value) -> UnreactResult<String> {
    // Get template string from name
    let template = match self.templates.get(name) {
      Some(s) => s,
      None => return Err(UnreactError::TemplateNotExist(name.to_string())),
    };

    // Create handlebars registry
    let mut reg = Handlebars::new();

    // Register all other templates as partials
    for (name, part) in &self.templates {
      if let Err(err) = reg.register_partial(name, part) {
        return Err(UnreactError::RegisterPartialFail(name.to_string(), err));
      }
    }

    // Register inbuilt partials
    for (name, part) in self.inbuilt_partials() {
      if let Err(err) = reg.register_partial(name, part) {
        return Err(UnreactError::RegisterInbuiltPartialFail(
          name.to_string(),
          err,
        ));
      }
    }

    // ? Remove `.clone` (2x) ? how ?
    let mut data = data.clone();
    if !self.globals.is_null() {
      merge_json(&mut data, self.globals.clone());
    }

    // Render template
    match reg.render_template(template, &data) {
      Ok(x) => Ok(x),
      Err(err) => Err(UnreactError::HandlebarsFail(name.to_string(), err)),
    }
  }

  /// Get inbuilt partials to register in `Unreact::render`
  fn inbuilt_partials(&self) -> Vec<(&'static str, String)> {
    vec![
      (
        // Base url for site
        "URL",
        if self.is_dev {
          format!("http://{}", server::ADDRESS)
        } else {
          self.url.to_string()
        },
      ),
      // Script for development
      // Is not registered if `dev_warning` in config is false
      (
        "DEV_SCRIPT",
        if self.is_dev && self.config.dev_warning {
          server::DEV_SCRIPT.to_string()
        } else {
          "".to_string()
        },
      ),
      // Simple link
      (
        "LINK",
        r#"<a href="{{>URL}}/{{to}}"> {{>@partial-block}} </a>"#.to_string(),
      ),
      // Simple style tag
      (
        "STYLE",
        r#"<link rel="stylesheet" href="{{>URL}}/styles/{{name}}.css" />"#.to_string(),
      ),
    ]
  }

  /// Open local server and listen
  fn listen() {
    server::listen();
  }

  /// Returns as error if any value of `config` are not valid directories
  ///
  /// Creates build directory
  fn check_dirs(config: &Config) -> UnreactResult<()> {
    // Collate directory names
    let dirs = vec![&config.templates, &config.public, &config.styles];

    // Loop directories that should exist
    for dir in dirs {
      // Check if directory exists
      let path = Path::new(dir);
      if !path.is_dir() {
        // return Err(Box::new(UnreactErrorOld(format!(
        //   "Directory `{dir}` does not exist"
        // ))));
        return Err(UnreactError::DirNotExist(dir.to_string()));
      }
    }

    // Remove build directory if exists
    if Path::new(&format!("./{}", config.build)).exists() {
      if let Err(err) = fs::remove_dir_all(format!("./{}", config.build)) {
        return Err(UnreactError::IoError(err, config.build.to_string()));
      };
    }

    // Create new build directory and generic subfolders
    let dirs = vec!["", "/styles", "/public"];
    for dir in dirs {
      if let Err(err) = fs::create_dir(format!("./{}{}", config.build, dir)) {
        return Err(UnreactError::IoError(err, config.build.to_string()));
      }
    }

    Ok(())
  }

  /// Load all templates in directory of `templates` property in `config`
  fn load_templates(config: &Config) -> UnreactResult<FileMap> {
    let mut templates = FileMap::new();
    load_filemap(&mut templates, &config.templates, "")?;
    Ok(templates)
  }

  /// Import all scss files in directory of `styles` property in `config`
  fn load_styles(config: &Config) -> UnreactResult<FileMap> {
    let mut styles = FileMap::new();
    load_filemap(&mut styles, &config.styles, "")?;
    Ok(styles)
  }
}
