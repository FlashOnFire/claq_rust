#![feature(test)]
extern crate test;

use std::fs::File;
use std::path::PathBuf;
use test::Bencher;
use lazy_regex::regex_replace_all;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelBridge};
use rayon::iter::ParallelIterator;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Article {
    pub pdfs: Vec<FileItem>,
    pub docx: Vec<FileItem>,
}

#[derive(Serialize, Deserialize)]
struct FileItem {
    pub name: String,
    pub content: String,
}

impl FileItem {
    pub fn count_words(&self) -> u32 {
        self.content.lines().par_bridge().into_par_iter().map(|line|
        line.split_terminator(|c: char| !c.is_alphanumeric())
            .fold(0, |acc, _| acc + 1)
        ).sum()
    }
}

fn main() {
    let mut articles: Vec<Article> = serde_json::from_reader(File::open(PathBuf::from("./articles.json")).unwrap()).unwrap();
    let total_article_count = articles.par_iter_mut().count();

    let (total_pdf_count, pdfs_wc, total_docx_count, docx_wc) = articles.par_iter_mut().map(|article: &mut Article| {
        // iterate over pdfs
        let pdfs_count = article.pdfs.par_iter().count();
        let pdfs_wc = article.pdfs.par_iter_mut().map(|file: &mut FileItem| {
            let wc = file.count_words();

            format_pdf(&mut file.content);

            wc
        }).sum();

        // iterate over docx
        let docxs_count = article.docx.par_iter().count();
        let docx_wc = article.docx.par_iter_mut().map(|file: &mut FileItem| {
            file.count_words()
        }).sum();

        (pdfs_count, pdfs_wc, docxs_count, docx_wc)
    }).reduce(|| (0_usize, 0, 0_usize, 0), |acc, e| (acc.0 + e.0, acc.1 + e.1, acc.2 + e.2, acc.3 + e.3));

    serde_json::to_writer(File::create("./articles_cleaned.json").unwrap(), &articles).unwrap();

    println!("Number of articles processed: {total_article_count}");
    println!("Total number of pdf items: {total_pdf_count}");
    println!("Total number of docx items: {total_docx_count}");
    println!("Total number of words processed: {}", pdfs_wc + docx_wc)
}


fn format_pdf(str: &mut String) {
    *str = regex_replace_all!(r"\n|\t", str, "").to_string();
    *str = regex_replace_all!(r" +", str, "").to_string();
    *str = regex_replace_all!(r#"\""#, str, "").to_string();
    *str = regex_replace_all!(r"((http|https):\/\/.*\.(jpg|jpeg|png|gif))", str, "").to_string();
    *str = regex_replace_all!(r"[^\\x00-\\x7F]", str, "").to_string();
    *str = regex_replace_all!(r"(^\\s+|\\s+$)", str, "").to_string();
}

#[bench]
fn bench(b: &mut Bencher) {
    b.iter(|| main());
}