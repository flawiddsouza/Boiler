extern crate toml;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate walkdir;

use std::fs::File;
use std::io::Read;
use std::env;
use std::collections::BTreeMap;
use std::path::Path;
use walkdir::WalkDir;
use std::fs;
use std::error::Error;
use std::process;

#[derive(Debug, Deserialize)]
struct Boilers {
    boilerplates: Vec<BTreeMap<String, String>>,
}

fn main() {
    let current_executable_file_path = match env::current_exe() {
        Err(err) => {
            println!("Boiler: Couldn't get current executable file path: {}", err.description());
            process::exit(1);
        },
        Ok(result) => result,
    };
    let current_executable_directory = match current_executable_file_path.parent() {
        None => {
            println!("Boiler: Couldn't get current executable directory");
            process::exit(1);
        },
        Some(result) => result,
    };
    let mut toml_file = match File::open(current_executable_directory.join("boilers.toml")) {
        Err(err) => {
            println!("Boiler: Unable to find boilers.toml: {}", err.description());
            process::exit(1);
        },
        Ok(result) => result,
    };
    let mut toml_string = String::new();
    match toml_file.read_to_string(&mut toml_string) {
        Err(err) => {
            println!("Boiler: Unable to read to string: {}", err.description());
            process::exit(1);
        },
        Ok(result) => result,
    };
    let toml_parsed: Boilers = match toml::from_str(&toml_string) {
        Err(err) => {
            println!("Boiler: Failed to parse toml: {}", err.description());
            process::exit(1);
        },
        Ok(result) => result,
    };
    // println!("{:#?}", toml_parsed); // see what was parsed - {:#?} is why Debug exists in "#[derive(Debug, Deserialize)]""

    let args: Vec<_> = env::args().collect();
    if args.len() > 1 {
        if args[1] == "ls" {  // boiler ls
            boiler_ls(&toml_parsed.boilerplates);
        } else if args[1] == "version" { // boiler version
            boiler_version();
        } else {
            // if the boilerplate access_name is 'ls' or 'version' or 'help', this will never be hit
            // hence they can never be used as boilerplate access_names
            // but even if they are, nothing happens because they're inaccessible
            // this prevents Boiler from breaking when there is a naming clash
            boiler(&args[1], &toml_parsed.boilerplates);
        }
    } else { // when no arguments are passed
        boiler_ls(&toml_parsed.boilerplates);
    }
}

fn boiler_ls(boilerplates: &Vec<BTreeMap<String, String>>) {
    println!("The available boilerplates are:");
    for boilerplate in boilerplates {
        println!("    {0}\n\t{1}", boilerplate["access_name"], boilerplate["name"]);
    }
}

fn boiler_version() {
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");
    println!("Boiler v{}", VERSION);
}

fn boiler_command_not_found() {
    println!("Boiler: No such command!");
    println!("Boiler: Try 'boiler ls' for available options");
}

fn boiler(arg: &String, boilerplates: &Vec<BTreeMap<String, String>>) {
    let mut match_found = false;
    for boilerplate in boilerplates {
        if arg.to_string() == boilerplate["access_name"] {
            match_found = true;
            if boilerplate["type"] == "local" {
                let path = Path::new(&boilerplate["path"]);
                if path.is_dir() {
                    println!("Boiler: Boilerplate path exists");
                    for entry in WalkDir::new(&path).into_iter().filter_map(|e| e.ok()) {
                        if entry.path().is_file() {
                            println!("Boiler: Creating {:?}", entry.file_name());
                            match File::create(entry.file_name()) {
                                Err(err) => {
                                    println!("Boiler: Unable to create file: {}", err.description());
                                    process::exit(1);
                                },
                                Ok(result) => result,
                            };
                            match fs::copy(entry.path(), entry.file_name()) {
                                Err(err) => {
                                    println!("Boiler: Unable to copy file: {}", err.description());
                                    process::exit(1);
                                },
                                Ok(result) => result,
                            };
                        }
                    }
                    println!("Boiler: Boiling done");
                } else {
                    println!("Boiler: Can't find the required files for {0} ({1})", boilerplate["access_name"], boilerplate["name"]);
                    println!("Boiler: Make sure the path is correct in boilers.toml");
                    println!("Boiler: Boiling Failed");
                }
            } else if boilerplate["type"] == "remote" {
                println!("Boiler: Not Implemented");
            }
        }
    }
    if !match_found {
        boiler_command_not_found();
    }
}