use serde_json::json;

use ssg::{App, AppOptions};

fn main() -> ssg::AppResult<()> {
  // Create interface object with default options
  let mut app = App::new(AppOptions::default())?;

  println!("{app:#?}");

  // return Ok(());

  // Create `/index.html` page using `index.hbs` template, test data
  app.index(&App::render("index", &json!({"test": 123}))?)?;

  // Create `/404.html` page using `error/not_found.hbs` template, test data
  app.not_found(&App::render("error/not_found", &json!({"test": 123}))?)?;

  // Create custom page at `/hello.html` using `hello.hbs` template, custom message
  app.page("hello", &App::render("hello", &json!({"msg": "Hello!"}))?)?;
  // Create custom page at `/hello/again.html` using `hello.hbs` template, different custom message
  app.page(
    "hello/again",
    &App::render("hello", &json!({"test": "Hello again!"}))?,
  )?;

  // Example data for 'dynamic' generation
  let posts = vec![
    ("example", "this is an example", "Monday"),
    ("other", "another example", "Wednesday"),
  ];
  // Loop data
  for (name, content, day) in posts {
    // Each data entry, create page with id, and 'dynamic' content
    app.page(
      &format!("post/{name}"),
      &App::render("post", &json!({ "content": content, "day": day }))?,
    )?;
  }

  if is_dev() {
    // Open dev server and listen
    app.listen()?;
  } else {
    // Compile files for production
    app.finish()?;
  }

  Ok(())
}

/// Check if `--dev` or `-d` argument was passed on run
fn is_dev() -> bool {
  let args = std::env::args().collect::<Vec<_>>();
  args.contains(&"--dev".to_string()) || args.contains(&"-d".to_string())
}
