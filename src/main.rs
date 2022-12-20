mod data;
use crate::data::bookmark::{Bookmarks, Child, Folder};
use clap::{arg, command, value_parser};
use notify::{
    event::{ModifyKind, RenameMode},
    Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use serde_json;
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{join, runtime::Handle, sync::mpsc::channel};

type FileMutex = Arc<Mutex<File>>;

async fn check_file(file_path: String, check_interval: Duration, output_file: FileMutex) {
    loop {
        let initial_metadata = fs::metadata(file_path.clone()).unwrap();
        let initial_modified_time = initial_metadata.modified().unwrap();

        tokio::time::sleep(check_interval).await;

        let current_metadata = fs::metadata(file_path.clone()).unwrap();
        let current_modified_time = current_metadata.modified().unwrap();

        if initial_modified_time != current_modified_time {
            let mut output_file = output_file.lock().unwrap();
            let file_content = fs::read_to_string(file_path.clone()).unwrap();
            let bookmarks: Bookmarks =
                serde_json::from_str(&file_content).expect("Expected Bookmarks struct");
            let captured_bookmarks = extract_bookmarks(&bookmarks.roots.bookmark_bar);
            output_file
                .write_all(
                    serde_json::to_string(&captured_bookmarks)
                        .unwrap()
                        .as_bytes(),
                )
                .unwrap();
        }
    }
}

async fn check_directory(directory_path: String, output_file: FileMutex) {
    let (tx, mut rx) = channel::<Event>(1);
    let target_file = PathBuf::from(format!("{}{}", directory_path, "Bookmarks"));
    let handle = Handle::current();

    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            handle.block_on(async { tx.send(res.unwrap()).await.unwrap() })
        },
        Config::default(),
    )
    .unwrap();

    watcher
        .watch(Path::new(&directory_path), RecursiveMode::Recursive)
        .unwrap();
    while let Some(event) = rx.recv().await {
        if event.kind == EventKind::Modify(ModifyKind::Name(RenameMode::Both))
            && event.paths.contains(&target_file)
        {
            let mut output_file = output_file.lock().unwrap();
            output_file
                .write_all(format!("{:?}\n", event).as_bytes())
                .unwrap();
        }
    }
}

fn extract_bookmarks(folder: &Folder) -> Vec<Child> {
    let mut bookmarks = Vec::new();
    for child in &folder.children {
        match child {
            Child::Folder(f) => {
                if f.name == "capture" {
                    let folder_bookmarks = extract_bookmarks(f);
                    bookmarks.extend(folder_bookmarks);
                }
            }
            Child::Bookmark { .. } => bookmarks.push(child.clone()),
        }
    }
    bookmarks
}

#[tokio::main]
async fn main() {
    // Parse command line arguments
    let matches = command!()
        .arg(arg!(file: -f <FILE>).value_parser(value_parser!(String)))
        .arg(arg!(directory: -d <DIRECTORY>).value_parser(value_parser!(String)))
        .get_matches();

    let file_arg = matches.get_one::<String>("file").unwrap();
    let dir_arg = matches.get_one::<String>("directory").unwrap();
    let file_path = format!("{}{}", dir_arg, file_arg);
    let dir_path = dir_arg.to_owned();
    let check_interval = std::time::Duration::from_secs(5);
    let file = fs::File::create("./test.txt").unwrap();
    let file_mutex = Arc::new(Mutex::new(file));
    let file_checker_mutex = Arc::clone(&file_mutex);

    let (_first, _second) = join!(
        tokio::spawn(async move {
            check_file(file_path, check_interval, file_checker_mutex).await;
        }),
        tokio::spawn(async move {
            check_directory(dir_path, file_mutex).await;
        })
    );
}
