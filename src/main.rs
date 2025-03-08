use doko::{levenshtein, run_task_with_timer};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use walkdir::WalkDir;

fn main() {
    let mut files: Vec<String> = vec![];

    run_task_with_timer("index", || {
        for entry in WalkDir::new("/home/haris/") {
            files.push(entry.unwrap().file_name().to_str().unwrap().to_string());
        }

        println!("Total files found: {}", files.len());
    });

    println!("----");

    let query = "main.cpp";

    run_task_with_timer("search", || {
        let filtered_files: Vec<&String> = files
            .par_iter()
            .filter(|&file| levenshtein(&file, &query) < 2)
            .collect();

        println!(
            "Files matching \"{}\" found: {}",
            query,
            filtered_files.len()
        );
    });
}
