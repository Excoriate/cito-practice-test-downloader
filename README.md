# 📄 CITO Practice Test Downloader

This Rust application downloads practice test documents from the CITO website, organizing them by year, exam, and document type. It scrapes the website for available documents and downloads them into a structured directory.

## 📋 Features

- 📅 Organizes documents by year and exam.
- 📂 Creates subdirectories for different document types (Opg, CV, Anv. CV).
- 🔄 Retries downloads up to 3 times in case of failure.
- ⏱️ Configurable timeout for HTTP requests.

## 🚀 Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) installed on your machine.

### Installation

1. Clone the repository:
    ```sh
    git clone https://github.com/yourusername/cito-practice-test-downloader.git
    cd cito-practice-test-downloader
    ```

2. Build the project:
    ```sh
    make build
    ```

### Usage

1. Run the downloader:
    ```sh
    make run
    ```

2. Enter the year for document download when prompted:
    ```sh
    Enter the year for document download:
    2023
    ```

3. Optionally, specify a period (1, 2, 3, 4) or use "all" to download documents for all periods:
    ```sh
    make run ARGS="--year 2023 --period 1"
    ```

### Directory Structure

The downloaded documents will be organized in the following structure:

```
./cito-practice-test-downloader
./cito-practice-test-downloader/2023
./cito-practice-test-downloader/2023/1
./cito-practice-test-downloader/2023/1/Opgave
./cito-practice-test-downloader/2023/1/Opgave/2023_1_Opgave_1.pdf
./cito-practice-test-downloader/2023/1/Opgave/2023_1_Opgave_2.pdf
