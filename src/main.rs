extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::env;
use std::fs;
use std::path::Path;
use std::thread;
use std::io::Write;

#[derive(Debug, Serialize, Deserialize)]
struct Media {
    id: String,
    #[serde(rename = "originalName")] original_name: String,
    mimetype: String,
    size: i64,
}

fn main() {
    let mut args = env::args();
    let _name = args.next();
    let media_url = args.next().expect("Usage: ./upload [media_url] [folder]");
    let folder = args.next().expect("Usage: ./upload [media_url] [folder]");
    let folder = Path::new(&folder);

    let mut file = fs::OpenOptions::new()
        .write(true)
        .append(false)
        .create(true)
        .open("results.json")
        .expect("Could not create/update results.json file");

    let files = fs::read_dir(folder).expect("Failed to read folder");

    let paths: Vec<std::path::PathBuf> = files
        .map(|file| {
            let file = file.unwrap();
            file.path()
        })
        .collect();

    let results: Vec<Vec<Media>> = paths
        .chunks(2)
        .map(|paths| {
            let url = media_url.clone();
            let paths = paths.to_owned();

            thread::spawn(move || upload(url, paths))
        })
        .map(|handle| handle.join().unwrap())
        .collect();

    let json = serde_json::to_string(&results).expect("Failed to convert upload results to JSON");

    match file.write_all(json.as_bytes()) {
        Ok(_) => println!("results file written"),
        Err(e) => println!("{}", e),
    }
}

fn upload(url: String, paths: Vec<std::path::PathBuf>) -> Vec<Media> {
    let client = reqwest::Client::new();
    let form = reqwest::multipart::Form::new();

    let params = paths.iter().fold(form, |form, ref path| {
        let name = path.file_name().unwrap().to_str().unwrap().to_owned();

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
