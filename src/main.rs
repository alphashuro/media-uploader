extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::env;
use std::fs::{self, File, OpenOptions};
use std::path::PathBuf;
use std::thread;

fn main() {
    let (media_url, folder) = get_args();
    let results_file: File = open_results_file();

    let files: Vec<PathBuf> = read_dir(folder);
    let results: Vec<Media> = upload_files(media_url, files);

    write(results_file, results)
}

fn write(file: File, data: Vec<Media>) {
    serde_json::to_writer_pretty(file, &data)
        .expect("Failed to convert data to JSON and write to file");
}

fn read_dir(folder: String) -> Vec<PathBuf> {
    let folder = PathBuf::from(&folder);

    fs::read_dir(folder)
        .expect("Failed to read folder")
        .map(|file| file.unwrap().path())
        .collect()
}

fn get_args() -> (String, String) {
    let mut args = env::args().skip(1);
    let media_url = args.next().expect("Usage: ./upload [media_url] [folder]");
    let folder = args.next().expect("Usage: ./upload [media_url] [folder]");

    (media_url, folder)
}

#[derive(Debug, Serialize, Deserialize)]
struct Media {
    id: String,
    #[serde(rename = "originalName")] original_name: String,
    mimetype: String,
    size: i64,
}

fn upload_files(media_url: String, files: Vec<PathBuf>) -> Vec<Media> {
    files
        .chunks(10)
        .map(|paths| {
            let url = media_url.clone();
            let paths = paths.to_owned();

            thread::spawn(move || upload(url, paths))
        })
        .flat_map(|handle| handle.join().unwrap().into_iter())
        .collect()
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

fn open_results_file() -> File {
    OpenOptions::new()
        .write(true)
        .append(false)
        .create(true)
        .open("results.json")
        .expect("Could not create/update results.json file")
}
