use std::{fs, path::Path};

use super::DEV_BUILD_DIR;

/// Local address with port to host dev server
pub const ADDRESS: &str = "127.0.0.1:8080";

/// Create server and listen on local port
///
/// Almost mimics GitHub Pages
///
/// Reads file on every GET request, however this should not be a problem for a dev server
pub fn listen() {
  use futures::Future;
  use hyper::service::service_fn_ok;
  use hyper::{Body, Method, Response, Server, StatusCode}; // For `map_error`

  // Setup and run the server
  let addr = ADDRESS.parse().unwrap();

  let router = || {
    service_fn_ok(|req| {
      // Check if is GET request
      if req.method() == &Method::GET {
        println!("{}", req.uri().path());

        // Convert request to possible filepaths
        let possible_files = possible_files_from_path(req.uri().path());
        for file in possible_files {
          // If file exists, and not directory
          if Path::new(&file).is_file() {
            println!("  - {file}");

            // Return body using contents of that file
            return Response::new(Body::from(
              fs::read_to_string(file).expect("Could not read file"),
            ));
          }
        }
      }

      // Default response
      let mut res = Response::new(Body::from("Not Found"));
      *res.status_mut() = StatusCode::NOT_FOUND;
      res
    })
  };

  let server = Server::bind(&addr).serve(router);

  println!("Listening on http://{ADDRESS}");

  hyper::rt::run(server.map_err(|e| {
    eprintln!("server error: {}", e);
  }));
}

/// Converts path from request into possible files to correspond to
///
/// If path ends with `.html`, returns path
///
/// Else returns path + `.html`, and path + `/index.html`
///
/// All file paths returned are relative to workspace directory, and include dev build path
fn possible_files_from_path(path: &str) -> Vec<String> {
  if path.ends_with(".html") {
    vec![format!("./{DEV_BUILD_DIR}/{}", path)]
  } else {
    vec![
      format!("./{DEV_BUILD_DIR}/{}.html", path),
      format!("./{DEV_BUILD_DIR}/{}/index.html", path),
    ]
  }
}
