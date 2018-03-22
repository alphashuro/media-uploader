extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate dotenv;

use dotenv::dotenv;
use std::env;
use std::env;
use std::fs;
use std::path::Path;
use std::thread;
use std::io::Write;

#[derive(Debug, Serialize, Deserialize)]
struct Media {
    id: String,
    #[serde(rename = "originalName")]
    original_name: String,
    mimetype: String,
    size: i64
}

fn main() {
    dotenv().ok();
    let mut args = env::args();

    let _name = args.next();
    let folder = args.next().expect("Usage: ./upload [folder]");
    let folder = Path::new(&folder);

    let files = fs::read_dir(folder).unwrap();

    let mut children = vec![];

    for (i, file) in files.enumerate() {
        let file = file.unwrap().path();
        println!("index {} file {:?}", i, file);

        children.push(thread::spawn(move || -> Media {
            let client = reqwest::Client::new();
            let params = reqwest::multipart::Form::new()
                .file("file", file).unwrap();

            let url = format!("{}/upload", dotenv!("MEDIA_URL"));

            let result: Media = client.post(url)
                .multipart(params)
                .send().unwrap()
                .json::<Vec<Media>>().unwrap()
                .pop().unwrap();

            println!("uploaded file {}, result={:?}", file, result);

            result
        }));
    }

    let mut results = vec![];

    for child in children {
        // collect each child thread's return-value
        match child.join() {
            Ok(result) => results.push(result),
            Err(e) => println!("Failed to upload {:?}", e)
        }
    }

    let mut file = fs::OpenOptions::new().write(true).append(false).create(true).open("results.json").unwrap();

    let json = serde_json::to_string(&results).unwrap();

    file.write_all(json.as_bytes());
}