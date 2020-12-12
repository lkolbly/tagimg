#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use clap::{App, Arg};
use log::*;
use rand::Rng;
use rocket::request::Form;
use rocket::State;
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::Template;

#[get("/hello/<name>/<age>")]
fn hello(name: String, age: u8) -> String {
    format!("Hello, {} year old named {}!", age, name)
}

#[derive(serde::Serialize)]
struct TemplateContext {
    imgname: String,
    classes: Vec<String>,
    files_left: usize,
}

struct Config {
    images: String,
    dataset: String,
    classes: Vec<String>,
}

fn render_template(config: &Config) -> Template {
    let files = std::fs::read_dir(&config.images).unwrap();
    let files: Vec<_> = files
        .map(|entry| {
            entry
                .unwrap()
                .path()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
        })
        .collect();

    if files.len() == 0 {
        panic!("All out of files to process!");
    }

    let mut rng = rand::thread_rng();
    let idx = rng.gen::<usize>() % files.len();
    let filename = &files[idx];

    let context = TemplateContext {
        imgname: filename.clone(),
        classes: config.classes.clone(),
        files_left: files.len(),
    };
    Template::render("index", &context)
}

#[get("/")]
fn index(config: State<Config>) -> Template {
    render_template(&config)
}

#[derive(FromForm)]
struct TaggedLabel {
    filename: String,
    tag: String,
}

#[post("/", data = "<tag>")]
fn tag(tag: Form<TaggedLabel>, config: State<Config>) -> Template {
    info!("Tagged {} as {}", tag.filename, tag.tag);
    std::fs::rename(
        format!("{}/{}", config.images, tag.filename),
        format!("{}/{}/{}", config.dataset, tag.tag, tag.filename),
    )
    .expect("Unable to move files!");
    render_template(&config)
}

fn main() {
    let matches = App::new("Quik Image Tagger")
        .author("Lane Kolbly")
        .about("Spins up a HTTP server you can use to quickly tag images")
        .arg(Arg::with_name("image_directory")
            .short("i")
            .long("images")
            .help("The path to a directory containing images to tag.")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("dataset")
            .short("o")
            .long("dataset")
            .help("Path to a directory containing classnames as subdirectores, each containing images.")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("classes")
            .short("c")
            .long("classes")
            .help("Comma-separated list of possible classes to tag images with. If not specified, auto-detects from the contents of the dataset directory.")
            .takes_value(true))
        .get_matches();

    let image_directory = matches.value_of("image_directory").unwrap();
    let dataset_directory = matches.value_of("dataset").unwrap();
    let classes: Vec<_> = match matches.value_of("classes") {
        Some(classes) => classes.split(",").map(|s| s.to_string()).collect(),
        None => std::fs::read_dir(dataset_directory)
            .expect("--classes not specified and dataset directory does not exist")
            .map(|entry| {
                entry
                    .unwrap()
                    .path()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string()
            })
            .collect(),
    };
    println!("Found classes {:?}", classes);

    let config = Config {
        images: image_directory.to_string(),
        dataset: dataset_directory.to_string(),
        classes,
    };

    rocket::ignite()
        .mount("/", routes![hello, index, tag])
        .mount("/images", StaticFiles::from(image_directory))
        .attach(Template::fairing())
        .manage(config)
        .launch();
}
