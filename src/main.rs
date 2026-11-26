use std::io::{Seek, Write};
use std::{env, ffi::OsStr, io::Read, path};
use std::fs::File;
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use regex::Regex;

fn main() {




    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args.len() > 3 {
        println!("Usage: {} <inputpath> <optionaloutputpath>", &args[0]);
        return;
    }
    let input_path = std::path::Path::new(&args[1]);
    let input_stem = input_path.file_stem().unwrap_or(OsStr::new(""));
    let input_ext = input_path.extension().unwrap_or(OsStr::new(""));
    let alsjkd = input_path.parent().unwrap_or(path::Path::new(""));
    let output_path_thing = format!("{0}_mutilated.{1}", input_stem.to_str().unwrap(), input_ext.to_str().unwrap());
    let output_path = alsjkd.join(std::path::Path::new(args.get(2).unwrap_or(&output_path_thing)));

    if (args.len() == 2) {
        println!("No output path provided, using default: {}", output_path.display());
    } else {
        println!("Output path provided: {}", output_path.display());
    }

    println!("Enter the version decimal (e.g., 4.70):");
    let mut version_decimal = String::new();
    let _ = std::io::stdin().read_line(&mut version_decimal).unwrap().to_string().trim_end();
    version_decimal = version_decimal.trim().to_string();

    println!("Enter the version semantic (e.g., 4.7.7):");
    let mut version_semver = String::new();
    let _ = std::io::stdin().read_line(&mut version_semver).unwrap().to_string().trim_end();
    version_semver = version_semver.trim().to_string();


    let mut files = read_zip_file(input_path).unwrap();
    mutilate_files(&mut files, version_decimal, version_semver);
    write_zip_file(&output_path, files).unwrap();
}
fn read_zip_file(path: &std::path::Path) -> Result<Vec<HashMap<Box<PathBuf>, Box<File>>>, Box<dyn std::error::Error>> {
    let file = std::fs::File::open(path)?;

    let mut archive = zip::ZipArchive::new(file)?;
    let mut buffer: Vec<HashMap<Box<PathBuf>, Box<File>>> = Vec::new();
    let tempdir = env::temp_dir().join(format!(
        "msczmutilate-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    std::fs::create_dir(&tempdir)?;

    for i in 0..archive.len() {

        let mut file = archive.by_index(i)?;

        let mut contents = Vec::new();
        file.read_to_end(&mut contents)?;

        let name = file.name()?.to_string();


        let temp_file_path = tempdir.join(name);

        if let Some(parent) = temp_file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut temp_file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&temp_file_path)?;

        temp_file.write_all(&contents)?;
        buffer.push(HashMap::from([(Box::new(temp_file_path), Box::new(temp_file))]));
    }
    Ok(buffer)
}
fn write_zip_file(path: &std::path::Path, files: Vec<HashMap<Box<PathBuf>, Box<File>>>) -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::File::create(path)?;
    let mut archive = zip::ZipWriter::new(file);
    for file_map in files {
        let (path, mut file) = file_map.into_iter().next().unwrap();
        let name = path.file_name().unwrap_or(OsStr::new("")).to_str().unwrap().to_string();

        archive.start_file(name, zip::write::SimpleFileOptions::default())?;
        let mut contents = Vec::new();
        file.seek(std::io::SeekFrom::Start(0))?;
        file.read_to_end(&mut contents)?;
        archive.write_all(&contents)?;
    }
    archive.finish()?;
    println!("Done! Mutilated mscz file written to: {}", path.display());
    Ok(())
}
fn mutilate_files(files: &mut Vec<HashMap<Box<PathBuf>, Box<File>>>, version_decimal: String, version_semver: String) {
    let version_decimal_regex = Regex::new(r#"(<museScore\b[^>]*\bversion=")[^"]*(")"#).unwrap();
    let version_semver_regex = Regex::new(r#"(<programVersion>)[^<]*?(</programVersion>)"#).unwrap();
    for file_map in files.iter_mut() {
        let (path, mut file) = file_map.iter_mut().next().unwrap();
        let name = path.file_name().unwrap_or(OsStr::new("")).to_str().unwrap().to_string();

        let mut contents = Vec::new();
        file.seek(std::io::SeekFrom::Start(0)).unwrap();
        file.read_to_end(&mut contents).unwrap();
        // Mutilate the contents here
        if name.ends_with(".mscx") || name == "score_style.mss" {
            // replace version in <museScore version="*"> with VERSION_DECIMAL
            let contents_str = String::from_utf8_lossy(&contents);
            let new_contents_str = version_decimal_regex.replace_all(
                &contents_str,
                format!("${{1}}{}${{2}}", version_decimal),
            );
            contents = new_contents_str.into_owned().into_bytes();
        }
        if name.ends_with(".mscx") {
            // replace version in <programVersion></programVersion> with VERSION_SEMVER

            let contents_str = String::from_utf8_lossy(&contents);
            let new_contents_str = version_semver_regex.replace_all(
                &contents_str,
                format!("${{1}}{}${{2}}", version_semver),
            );
            contents = new_contents_str.into_owned().into_bytes();
        }
        file.seek(std::io::SeekFrom::Start(0)).unwrap();
        file.write_all(&contents).unwrap();
    }
}