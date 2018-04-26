extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;

fn main() {
    let mut args = env::args();
    let _name = args.next();
    let media_url = args.next().expect("Usage: ./upload [media_url] [folder]");
    let folder = args.next().expect("Usage: ./upload [media_url] [folder]");
    let folder = Path::new(&folder);

    let results_file = fs::OpenOptions::new()
        .write(true)
        .append(false)
        .create(true)
        .open("results.json")
        .expect("Could not create/update results.json file");

    let files: Vec<PathBuf> = fs::read_dir(folder)
        .expect("Failed to read folder")
        .map(|file| file.unwrap().path())
        .collect();

    let results: Vec<Vec<Media>> = files
        .chunks(10)
        .map(|paths| {
            let url = media_url.clone();
            let aths = paths.to_owned();

            thread::spawn(move || upload(url, paths))
        })
        .map(|handle| handle.join().unwrap())
        .collect();

    serde_json::to_writer_pretty(results_file, &results)
        .expect("Failed to convert upload results to JSON and write to file");
}

#[derive(Debug, Serialize, Deserialize)]
struct Media {
    id: String,
    #[serde(rename = "originalName")] original_name: String,
    mimetype: String,
    size: i64,
}

fn upload(url: String, paths: Vec<PathBuf>) -> Vec<Media> {
    let client = reqwest::Client::new();
    let form = reqwest::multipart::Form::new();

    let params = paths.iter().fold(form, |form, ref path| {
        let name = path.file_name().unwrap().to_str().unwrap().to_owned();

        println!("{}", name);

        form.file(name, path).unwrap()
    });

    let url = format!("{}/upload", url);

    let result: Vec<Media> = client
        .post(url.as_str())
        .multipart(params)
        .send()
        .expect("Could not read response from media service")
        .json()
        .expect("Response from media service is in an incorrect format");

    println!("uploaded {:?}, result={:?}", paths, result);

    result
}
