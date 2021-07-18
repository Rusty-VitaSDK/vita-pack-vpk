// #![allow(unused)]

use clap::ArgMatches;
use std::{
    fs::File,
    path::{Path, PathBuf},
};
use zip::{write::FileOptions, CompressionMethod::Stored, ZipWriter};

/// Used to store file or folder to be compressed into vpk
struct AddList {
    src: PathBuf,
    dst: String,
}

/// Default VPK filename
const DEFAULT_OUTPUT_FILE: &str = "output.vpk";
const DEFAULT_SFO_VPK_PATH: &str = "sce_sys/param.sfo";
const DEFAULT_EBOOT_VPK_PATH: &str = "eboot.bin";

fn main() {
    use clap::{App, Arg};

    let addlist_vec: Vec<AddList>;
    let vpk_path: &Path;
    let arg_matches: ArgMatches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .arg(
            Arg::with_name("sfo")
                .short("s")
                .long("sfo")
                .value_name("param.sfo")
                .help("Sets the param.sfo file")
                .validator(check_file)
                .takes_value(true)
                .required(true)
                .display_order(1),
        )
        .arg(
            Arg::with_name("eboot")
                .short("b")
                .long("eboot")
                .value_name("eboot.bin")
                .help("Sets the eboot.bin file")
                .validator(check_file)
                .takes_value(true)
                .required(true)
                .display_order(2),
        )
        .arg(
            Arg::with_name("add")
                .short("a")
                .long("add")
                .value_name("src=dst")
                .help("Adds the file or directory src to the vpk as dst")
                .validator(check_add)
                .multiple(true)
                .takes_value(true)
                .display_order(3),
        )
        .arg(
            Arg::with_name("vpk")
                .help("Name and path to the new .vpk file")
                .default_value(DEFAULT_OUTPUT_FILE)
                .index(1),
        )
        .get_matches();
    addlist_vec = build_list(&arg_matches);
    vpk_path = Path::new(arg_matches.value_of("vpk").unwrap_or_default());
    pack_vpk(addlist_vec, vpk_path);
}

fn check_file(file: String) -> Result<(), String> {
    let file_path = Path::new(&file);
    if !file_path.exists() {
        Err(String::from("File doesn't exist!"))
    } else if !file_path.is_file() {
        Err(String::from("Given path is not a valid file!"))
    } else {
        Ok(())
    }
}

fn check_add(var: String) -> Result<(), String> {
    if var.contains("=") && var.len() >= 3 {
        Ok(())
    } else {
        Err(String::from("Need <src=dst>. With src the source folder or path and dst where it should be in the vpk archive."))
    }
}

fn build_list(arg_matches: &ArgMatches) -> Vec<AddList> {
    let sfo_path: &Path;
    let eboot_path: &Path;
    let mut addlist_vec: Vec<AddList>;

    // Get sfo and eboot path from Clap Arg matches
    sfo_path = Path::new(arg_matches.value_of("sfo").unwrap());
    eboot_path = Path::new(arg_matches.value_of("eboot").unwrap());

    // Create our addlist Vector and push sfo and eboot addlists
    addlist_vec = Vec::new();
    addlist_vec.push(make_add_list(sfo_path, String::from(DEFAULT_SFO_VPK_PATH)));
    addlist_vec.push(make_add_list(eboot_path, String::from(DEFAULT_EBOOT_VPK_PATH)));

    // Check if add options are present, parse them, create and addlist and add
    // them to the AddList Vector
    if arg_matches.is_present("add") {
        for element in arg_matches.values_of("add").unwrap() {
            addlist_vec.push(parse_add(element));
        }
    }

    addlist_vec
}

fn make_add_list(src_path: &Path, dst_path: String) -> AddList {
    if !src_path.exists() {
        println!(
            "[ERR] Given file or folder doesn't exists: {}",
            src_path.to_str().unwrap()
        );
        std::process::exit(exitcode::NOINPUT);
    }
    AddList {
        src: src_path.to_path_buf(),
        dst: dst_path,
    }
}

fn parse_add(arg_add: &str) -> AddList {
    let splitted_arg_add: Vec<&str> = arg_add.split("=").collect();
    let src_path: &Path = Path::new(splitted_arg_add[0]);
    let dst_str: String = String::from(splitted_arg_add[1]);

    make_add_list(src_path, dst_str)
}

fn make_file(file_path: &Path) -> File {
    match File::create(file_path) {
        Ok(file) => file,
        Err(error) => {
            println!(
                "error: Unable to make the {:?} file : {:?}",
                file_path.to_str(),
                error
            );
            std::process::exit(exitcode::CANTCREAT);
        }
    }
}

fn pack_vpk(addlist: Vec<AddList>, vpk_path: &Path) {
    // Variable that will allow to create and write to our new vpk file
    let vpk_file: File;
    // Variable that will manage ZipWriter (Zip Archive Generator) to write our vpk file
    let mut vpk_writer: ZipWriter<File>;
    let options: FileOptions;

    vpk_file = make_file(vpk_path);
    vpk_writer = ZipWriter::new(vpk_file);
    options = FileOptions::default()
        .compression_method(Stored)
        .unix_permissions(0o755);

    for pair in addlist {
        match vpk_writer.start_file(&pair.dst, options) {
            Ok(()) => None,
            Err(error) => println!("Error: {}", error)
        }
        // println!("SRC : {:?}", pair.src);
        // println!("DST : {}\n", pair.dst);
    }
}
