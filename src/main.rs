extern crate reqwest;
extern crate select;

use chrono::prelude::*;
use select::document::Document;
use select::predicate::Name;
use std::collections::hash_set::HashSet;
use std::io::prelude::*;

// 1. Open current_cats file
// 2. Run function
// 3. Diff current_cats file contents + function results
// 4. Write new line in diff_file date + new/removed cats
// 5. Overwrite current_cats with function results

#[tokio::main]
async fn main() {
    let url_path = std::path::Path::new("url.txt");
    let mut url = String::new();
    std::fs::File::open(url_path)
        .unwrap()
        .read_to_string(&mut url)
        .unwrap();
    let future = cats(&url);
    future.await;
}

async fn cats(url: &str) {
    let cat_file_path = std::path::Path::new("current_cats.txt");
    let diff_file_path = std::path::Path::new("cats_diff.txt");
    let cats_list_file = std::fs::File::open(cat_file_path).unwrap();
    let old_cat_set: HashSet<String> = std::io::BufReader::new(cats_list_file)
        .lines()
        .map(|x| x.unwrap())
        .collect();

    let resp = reqwest::get(url).await.unwrap().text().await.unwrap();

    let pattern = regex::Regex::new(r"&n=(.+)").unwrap();
    let r: &str = &resp;
    let new_cat_set = Document::from(r)
        .find(Name("a"))
        .filter_map(|n| n.attr("href"))
        .flat_map(|link| pattern.captures_iter(link))
        .map(|result| (&result[1]).to_string())
        .collect::<HashSet<String>>();
    let mut cats_list_file = std::fs::File::create(cat_file_path).unwrap();
    for cat in new_cat_set.iter() {
        cats_list_file.write_all(cat.as_bytes()).unwrap();
        cats_list_file.write_all("\n".as_bytes()).unwrap();
    }
    let mut diff_file = std::fs::OpenOptions::new()
        .append(true)
        .open(diff_file_path)
        .unwrap();

    let current_time = Local::now();
    let formatted_time = current_time.format("%y.%m.%d");
    diff_file
        .write_all(format!("on {}:\n", formatted_time).as_bytes())
        .unwrap();
    for cat in old_cat_set.iter() {
        if !new_cat_set.contains(cat) {
            diff_file
                .write_all(format!("- removed {}\n", cat).as_bytes())
                .unwrap();
        }
    }

    for cat in new_cat_set.iter() {
        if !old_cat_set.contains(cat) {
            diff_file
                .write_all(format!("- added {}\n", cat).as_bytes())
                .unwrap();
        }
    }
}
