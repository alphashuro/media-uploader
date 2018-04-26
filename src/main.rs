#[macro_use]
extern crate clap;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate toml;

use std::io::{Read, Write};
use std::env;
use std::fs::{self, File, OpenOptions};
use std::path::PathBuf;
use std::thread;

fn main() {
    let matches = clap_app!(Uploader =>
        (version: "1.0")
        (author: "Alpha Shuro")
        (about: "Uploads media to the media service a la bluerobot")
        (@arg folder: -f --folder +takes_value "Folder to upload") 
        (@subcommand set =>
            (about: "Sets config options")
            (version: "1.0")
            (@arg url: -u --url +required +takes_value "Media Service URL")
        )
    ).get_matches();

    if let Some(folder) = matches.value_of("folder") {
        let config = read_config_file();
        let media_url = config.media_url;

        let folder = matches.value_of("folder").unwrap();
        let results_file: File = open_results_file();

        let files: Vec<PathBuf> = read_dir(folder.to_owned());
        let results: Vec<Media> = upload_files(media_url, files);

        write(results_file, results);
    }

    if let Some(matches) = matches.subcommand_matches("set") {
        if matches.is_present("url") {
            let url = matches.value_of("url").unwrap();
            let config = Config {
                media_url: url.to_owned(),
            };
            set_config(config);
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    media_url: String,
}

fn get_config_path() -> PathBuf {
    let (_, home_folder) = env::vars()
        .find(|&(ref key, _)| key == "HOME")
        .expect("HOME not defined!!");

    let mut path = PathBuf::from(home_folder);
    path.push(".uploader");

    path
}

fn read_config_file() -> Config {
    let path = get_config_path();
    let mut file = File::open(path).expect("Could not open config file");

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Could not read config file");

    toml::from_str(&contents).expect("Failed to parse config as toml")
}

fn set_config(config: Config) {
    let path = get_config_path();
    let toml = toml::to_string(&config).unwrap();

    let mut f = File::create(path).expect("Could not create config file");
    f.write_all(toml.as_bytes())
        .expect("Could not write config file");
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

fn get_args() -> String {
    let mut args = env::args().skip(1);
    let folder = args.next().expect("Usage: ./upload [folder]");

    folder
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
