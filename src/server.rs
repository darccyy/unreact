use http::{Method, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use std::{convert::Infallible, fs, path::Path};

use crate::DEV_BUILD_DIR;

//TODO Make better!!!
pub type UnreactDevError = String;

/// Local address with port to host dev server
pub const ADDRESS: &str = "127.0.0.1:8080";

/// Partial for hot reloading document in development
pub const DEV_SCRIPT: &str = r#"
  <script>
    console.warn("This document is in *development mode*");
  </script>
"#;

/// Create server and listen on local port
///
/// Almost mimics GitHub Pages
///
/// Reads file on every GET request, however this should not be a problem for a dev server
pub fn listen() {
  // Start `tokio` runtime (without macro)
  tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()
    .expect("Failed building the Runtime")
    .block_on(async {
      // Create service for router
      let make_svc = make_service_fn(|_| async {
        return Ok::<_, Infallible>(service_fn(router));
      });

      // Create server
      let addr = ADDRESS.parse().expect("Invalid IP address");
      let server = Server::bind(&addr).serve(make_svc);

      // Start server
      println!("Listening on http://{}", addr);
      server.await?;

      Ok::<_, hyper::Error>(())
    })
    .expect("Error in Runtime");
}

/// Route path to read and return file
async fn router(req: Request<Body>) -> Result<Response<Body>, Infallible> {
  // Check if is GET request
  if req.method() == Method::GET {
    // Return corresponding file as body if exists
    if let Some(file) = get_best_possible_file(req.uri().path()) {
      return Ok(Response::new(file));
    }
  }

  // 404 page
  Ok(
    Response::builder()
      .status(StatusCode::NOT_FOUND)
      .body(Body::from(
        // If custom 404 page is defined
        if let Some(file) = get_best_possible_file("404") {
          // Custom 404 page using request `/404`
          return Ok(
            Response::builder()
              .status(StatusCode::NOT_FOUND)
              .body(Body::from(file))
              .unwrap(),
          );
        } else {
          // Fallback 404 response
          "404 - File not found. Custom 404 page not found.".to_string()
        },
      ))
      .unwrap(),
  )
}

/// Loops through files in `possible_files_from_path` to find best file match
///
/// Returns `None` if no file was founds
///
/// Returns as `Option<Body>`, to allow non-UTF-8 file formats (such as images)
///
/// Panics if file exists, but was unable to be read
fn get_best_possible_file(path: &str) -> Option<Body> {
  // Convert request to possible filepaths
  let possible_files = possible_files_from_path(path);
  for file in &possible_files {
    let file = &format!("./{DEV_BUILD_DIR}/{file}");
    // If file exists, and not directory
    if Path::new(file).is_file() {
      // Returns file content as `Body`
      // Automatically parses to string, if is valid UTF-8, otherwise uses buffer
      return Some(Body::from(
        fs::read(file).expect(&format!("Could not read file '{file}'")),
      ));
    }
  }
  None
}

/// Converts path from request into possible files to correspond to
///
/// If path ends with `.html`, or starts with `/styles` or `/public`, returns path, unchanged
///
/// Else returns path + `.html`, and path + `/index.html`
///
/// All file paths returned are relative to workspace directory, and include dev build path
fn possible_files_from_path(path: &str) -> Vec<String> {
  if path.ends_with(".html") || path.starts_with("/styles") || path.starts_with("/public") {
    vec![path.to_string()]
  } else {
    vec![
      path.to_string(),
      path.to_string() + ".html",
      path.to_string() + "/index.html",
    ]
  }
}
