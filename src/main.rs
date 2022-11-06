use serde_json::json;

use unreact::{Unreact, Config};

const URL: &str = "https://darccyy.github.io/unreact";

fn main() -> unreact::UnreactResult<()> {
  // Example data for 'dynamic' generation
  let posts = vec![
    ("example", "this is an example", "Monday"),
    ("other", "another example", "Wednesday"),
  ];

  // Create interface object with default options
  let mut app = Unreact::new(
    Config {
      ..Config::github_pages()
    },
    is_dev(),
    URL,
  )?;

  // Register global variables
  app.set_globals(json!({"my_global": ":) hello from global!"}))?;

  // Create `/index.html` page using `index.hbs` template, test data
  app.index("index", &json!({"test": 123, "posts": posts}))?;

  // Create `/404.html` page using `error/not_found.hbs` template, test data
  app.not_found("error/not_found", &json!({"test": 123}))?;

  // Create a page at `/plain.html` with no template (plain)
  app.page_plain(
    "plain",
    "This was created without a template. <em>This should be italics</em>",
  )?;

  // Create custom page at `/hello.html` using `hello.hbs` template, custom message
  app.page("hello", "hello", &json!({"msg": "Hello!"}))?;
  // Create custom page at `/hello/again.html` using `hello.hbs` template, different custom message
  app.page("hello/again", "hello", &json!({"msg": "Hello again!"}))?;

  // Loop data
  for (name, content, day) in posts {
    // Each data entry, create page with id, and 'dynamic' content
    app.page(
      &format!("post/{name}"),
      "post",
      &json!({ "content": content, "day": day }),
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
