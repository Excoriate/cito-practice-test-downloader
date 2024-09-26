use anyhow::{Context, Result};
use clap::Parser;
use reqwest::blocking::{Client, ClientBuilder};
use scraper::{Html, Selector};
use std::fs;
use std::io::copy;
use std::path::Path;
use std::time::Duration;
use url::Url;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    year: u32,
    #[arg(short, long, default_value = "all")]
    period: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let client = create_client()?;

    let periods = if args.period == "all" {
        vec!["1", "2", "3", "4"]
    } else {
        vec![args.period.as_str()]
    };

    for period in periods {
        let base_url = format!(
            "https://www2.cito.nl/vo/ce/ex{}_havovwo/vwo-tv{}.htm",
            args.year, period
        );

        match process_period(&client, &base_url, &args.year, period) {
            Ok(_) => println!("Period {} processed successfully.", period),
            Err(e) => println!("Period {} not found for year {}: {}", period, args.year, e),
        }
    }

    println!("All periods processed.");
    Ok(())
}

fn create_client() -> Result<Client> {
    ClientBuilder::new()
        .timeout(Duration::from_secs(30))
        .build()
        .context("Failed to create HTTP client")
}

fn process_period(client: &Client, base_url: &str, year: &u32, period: &str) -> Result<()> {
    let response = client.get(base_url).send()?.text()?;
    let document = Html::parse_document(&response);

    let exam_selector = Selector::parse("tr").unwrap();
    let exams = document.select(&exam_selector);

    let year_dir = Path::new("downloads").join(year.to_string()).join(period);
    fs::create_dir_all(&year_dir).context("Failed to create year/period directory")?;

    for exam in exams {
        if let Some(exam_name_elem) = exam.select(&Selector::parse("td:first-child").unwrap()).next() {
            let exam_name = exam_name_elem.text().collect::<String>().trim().to_string();
            if !exam_name.is_empty() {
                println!("Processing exam: {}", exam_name);
                process_exam(client, base_url, &year_dir, &exam_name, &exam)?;
            }
        }
    }

    Ok(())
}

fn process_exam(client: &Client, base_url: &str, year_dir: &Path, exam_name: &str, exam_row: &scraper::element_ref::ElementRef) -> Result<()> {
    let folder_path = year_dir.join(sanitize_filename(exam_name));
    fs::create_dir_all(&folder_path).context("Failed to create exam folder")?;

    let opg_selector = Selector::parse("td:nth-child(2) a").unwrap();
    let cv_selector = Selector::parse("td:nth-child(4) a").unwrap();
    let anv_cv_selector = Selector::parse("td:nth-child(6) a").unwrap();

    download_files(client, base_url, &folder_path, exam_row, &opg_selector, "Opg")?;
    download_files(client, base_url, &folder_path, exam_row, &cv_selector, "CV")?;
    download_files(client, base_url, &folder_path, exam_row, &anv_cv_selector, "Anv. CV")?;

    Ok(())
}

fn download_files(client: &Client, base_url: &str, folder_path: &Path, exam_row: &scraper::element_ref::ElementRef, selector: &Selector, file_type: &str) -> Result<()> {
    let subdir_path = folder_path.join(file_type);
    fs::create_dir_all(&subdir_path).context(format!("Failed to create {} subdirectory", file_type))?;

    for link in exam_row.select(selector) {
        if let Some(href) = link.value().attr("href") {
            let file_url = Url::parse(base_url)?.join(href)?;
            let file_name = file_url.path_segments().and_then(|segments| segments.last()).unwrap_or("unknown");
            download_file_with_retry(client, &file_url, &subdir_path, file_name)?;
        }
    }

    Ok(())
}

fn download_file_with_retry(client: &Client, file_url: &Url, folder_path: &Path, file_name: &str) -> Result<()> {
    const MAX_RETRIES: u32 = 3;
    let mut retries = 0;

    while retries < MAX_RETRIES {
        match download_file(client, file_url, folder_path, file_name) {
            Ok(_) => return Ok(()),
            Err(e) => {
                retries += 1;
                if retries == MAX_RETRIES {
                    return Err(e);
                }
                println!("Retry {} for {}", retries, file_url);
                std::thread::sleep(Duration::from_secs(2));
            }
        }
    }

    Ok(())
}

fn download_file(client: &Client, file_url: &Url, folder_path: &Path, file_name: &str) -> Result<()> {
    let response = client.get(file_url.as_str()).send()?;
    if response.status().is_success() {
        let content_type = response.headers().get("content-type")
            .and_then(|ct| ct.to_str().ok())
            .unwrap_or("");

        let extension = if content_type.contains("pdf") {
            "pdf"
        } else if content_type.contains("msword") || content_type.contains("vnd.openxmlformats-officedocument.wordprocessingml.document") {
            "doc"
        } else {
            "unknown"
        };

        let file_path = folder_path.join(format!("{}.{}", sanitize_filename(file_name), extension));
        let mut file = fs::File::create(&file_path).context("Failed to create file")?;
        copy(&mut response.bytes()?.as_ref(), &mut file).context("Failed to copy content to file")?;
        println!("Downloaded: {}", file_path.display());
    } else {
        println!("Failed to download: {} (Status: {})", file_url, response.status());
    }
    Ok(())
}

fn sanitize_filename(filename: &str) -> String {
    filename.replace(|c: char| !c.is_alphanumeric() && c != ' ', "_")
}
