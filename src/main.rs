use clap::Parser;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use async_compression::Level;
use async_compression::tokio::write::GzipEncoder;

#[derive(Parser)]
struct Cli {
	input_dir: PathBuf,
	#[arg(long)]
	silent: bool
}

#[tokio::main]
async fn main() {
	let args = Cli::parse();
	recursive_compress(args.input_dir, args.silent).await;
}

#[async_recursion::async_recursion]
async fn recursive_compress(path: PathBuf, silent: bool) {
	let mut handles = vec![];
	let mut read_dir = fs::read_dir(path).await.unwrap();

	while let Some(dir_entry) = read_dir.next_entry().await.unwrap() {
		let file_type = dir_entry.file_type().await.unwrap();
		let path = dir_entry.path();

		if file_type.is_dir() {
			handles.push(tokio::spawn(async move {
				recursive_compress(path, silent).await;
			}));
		} else if file_type.is_file() {
			if let Some(extension) = path.extension() {
				if extension == "gz" {
					continue;
				}
			}

			handles.push(tokio::spawn(async move {
				compress_file(path, silent).await;
			}));
		}
	}

	for handle in handles {
		handle.await.unwrap();
	}
}

async fn compress_file(path: PathBuf, silent: bool) {
	let mut file_name = path.file_name().unwrap().to_os_string();
	file_name.push(".gz");
	let compress_to = path.with_file_name(file_name);

	let mut input_file = fs::File::open(path).await.unwrap();

	if let Ok(output_file) = fs::File::create(&compress_to).await {
		let mut input_buf = vec![];
		input_file.read_to_end(&mut input_buf).await.unwrap();

		let mut gzip_encoder = GzipEncoder::with_quality(output_file, Level::Best);
		gzip_encoder.write_all(&input_buf).await.unwrap();
		gzip_encoder.shutdown().await.unwrap();

		if !silent {
			println!("successfully compressed {compress_to:?}");
		}
	} else {
		println!("WARNING: skipping {compress_to:?} because it could not be opened");
	}
}
