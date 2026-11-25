use std::io::{Seek, Write};
use std::{env, ffi::OsStr, io::Read, path};
use std::fs::File;
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
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
    dbg!(&output_path);
    dbg!(&input_path);
    let files = read_zip_file(input_path).unwrap();
    write_zip_file(&output_path, files).unwrap();
}
fn read_zip_file(path: &std::path::Path) -> Result<Vec<HashMap<Box<PathBuf>, Box<File>>>, Box<dyn std::error::Error>> {
    let file = std::fs::File::open(path)?;
    dbg!(&file);
    let mut archive = zip::ZipArchive::new(file)?;
    let mut buffer: Vec<HashMap<Box<PathBuf>, Box<File>>> = Vec::new();
    let tempdir = env::temp_dir().join(format!(
        "msczmutilate-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    std::fs::create_dir(&tempdir)?;
    dbg!(&archive.len());
    for i in 0..archive.len() {
        dbg!(i);
        let mut file = archive.by_index(i)?;
        dbg!(&file.name());
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)?;
        dbg!(&contents.len());
        let name = file.name()?.to_string();
        dbg!(&name);
        dbg!(&tempdir.join(name.clone()));
        let temp_file_path = tempdir.join(name);
        dbg!(&temp_file_path);
        if let Some(parent) = temp_file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut temp_file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&temp_file_path)?;
        dbg!(&temp_file);
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
        dbg!(&name);
        archive.start_file(name, zip::write::SimpleFileOptions::default())?;
        let mut contents = Vec::new();
        file.seek(std::io::SeekFrom::Start(0))?;
        file.read_to_end(&mut contents)?;
        archive.write_all(&contents)?;
    }
    archive.finish()?;
    Ok(())
}