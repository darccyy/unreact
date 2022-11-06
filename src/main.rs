use serde_json::json;

use unreact::{App, AppConfig};

const URL: &str = "https://darccyy.github.io/unreact";

fn main() -> unreact::AppResult<()> {
  // Example data for 'dynamic' generation
  let posts = vec![
    ("example", "this is an example", "Monday"),
    ("other", "another example", "Wednesday"),
  ];

  // Create interface object with default options
  let mut app = App::new(AppConfig::github_pages(), is_dev(), URL)?;

  // Create `/index.html` page using `index.hbs` template, test data
  app.index(&app.render("index", &json!({"test": 123, "posts": posts}))?)?;

  // Create `/404.html` page using `error/not_found.hbs` template, test data
  app.not_found(&app.render("error/not_found", &json!({"test": 123}))?)?;

  // Create custom page at `/hello.html` using `hello.hbs` template, custom message
  app.page("hello", &app.render("hello", &json!({"msg": "Hello!"}))?)?;
  // Create custom page at `/hello/again.html` using `hello.hbs` template, different custom message
  app.page(
    "hello/again",
    &app.render("hello", &json!({"msg": "Hello again!"}))?,
  )?;

  // Loop data
  for (name, content, day) in posts {
    // Each data entry, create page with id, and 'dynamic' content
    app.page(
      &format!("post/{name}"),
      &app.render("post", &json!({ "content": content, "day": day }))?,
    )?;
  }

  // Compile files, host if in dev mode
  app.finish()?;
  println!("Compiled successfully.");

  Ok(())
}

/// Check if `--dev` or `-d` argument was passed on run
fn is_dev() -> bool {
  let args = std::env::args().collect::<Vec<_>>();
  args.contains(&"--dev".to_string()) || args.contains(&"-d".to_string())
}
