use argparse::{ArgumentParser, Store, StoreTrue};
use colored::Colorize;
use std::{fs, path::PathBuf};

const HANDLED_EXTENSIONS: &'static [&'static str] = &[
    "txt", "py", "pyw", "c", "cpp", "rs", "bat", "cmd", "toml", "md", "log",
];

#[derive(Default, Clone)]
struct Options {
    silent: bool,
    path: String,
    find: bool,
    str_to_find: String,
}
impl Options {
    fn update(&mut self) {
        self.find = self.str_to_find != "";
        self.path = match self.path.as_str() {
            "" => {
                let p: String = PathBuf::from(".")
                    .canonicalize()
                    .unwrap()
                    .display()
                    .to_string();
                p[4..p.len()].to_string()
            }
            _ => self.path.clone(),
        };
        if self.path == "" {
            let mut p: String = PathBuf::from(".")
                .canonicalize()
                .unwrap()
                .display()
                .to_string();
            p = p[4..p.len()].to_string();

            self.path = p
        }
    }
    fn show(&self) -> String {
        let mut msg = String::new();
        msg = String::from(format!(
            "{}\n{}",
            msg,
            format!("Selected path is: {}", self.path)
        ));
        msg.push_str(match self.silent {
            true => "\nSilent mode is on.",
            false => "\nSilent mode is off.",
        });

        msg = String::from(format!(
            "{}\n{}",
            msg,
            match self.find {
                true => {
                    format!("Search system will track: '{}'.", self.str_to_find)
                }
                false => String::from("Search system is desactivated."),
            }
        ));

        msg
    }
    fn clone_but_change_path(&self, path: String) -> Self {
        let mut new = self.clone();
        new.path = path;
        new
    }
}

#[derive(Default, Debug)]
struct File {
    path: String, // pathbuf ?
    times: i32,
}

impl File {
    fn new(path: String) -> Self {
        Self { path, times: 0 }
    }
}

#[derive(Default, Debug)]
struct Output {
    positively_searched_files: Vec<File>,
}

impl Output {
    fn update(&mut self, other: Self) {
        self.positively_searched_files
            .extend(other.positively_searched_files)
    }
    fn from(file: File) -> Self {
        let positively_searched_files = match file.times > 0 {
            true => vec![file],
            false => vec![],
        };
        Self {
            positively_searched_files,
        }
    }
    fn display(&self, options: Options) -> String {
        // print("\n\n============================================================")
        // for i in ll.filefound:
        //     print(i)
        // print("Il y as {} fichiers contennant le mot clé '{}'".format(
        //     len(ll.filefound), ll.SearchedSentence))
        // print("============================================================")

        let line_size = 60;

        let mut msg = String::new();
        let mut total_times = 0;

        msg.push_str(&"=".repeat(line_size));
        for file in self.positively_searched_files.iter() {
            // times = file.times:
            total_times += file.times;
            let times_repeat = clamp(3 - file.times.to_string().len(), 0, 100);
            let file_msg = match file.times > 1 {
                true => format!(
                    "found: {}{} times in: {}",
                    file.times,
                    " ".repeat(times_repeat),
                    file.path
                ),
                false => format!(
                    "found: {}{} time in: {}",
                    file.times,
                    " ".repeat(times_repeat),
                    file.path
                ),
            };
            msg = format!("{}\n{}", msg, file_msg)
        }
        msg.push_str("\n");
        msg.push_str(&"=".repeat(line_size));
        if options.find {
            msg = format!(
                "{}\n{}",
                msg,
                format!(
                    "Keyword: '{}' found {} times in {} files",
                    options.str_to_find,
                    total_times,
                    self.positively_searched_files.len()
                )
            )
        }

        msg
    }
}
fn clamp<T: std::cmp::PartialOrd>(nbr: T, min: T, max: T) -> T {
    if nbr < min {
        min
    } else if nbr > max {
        max
    } else {
        nbr
    }
}

fn search_folder(options: Options) -> Output {
    if !options.silent {
        println!(
            "{}",
            format!("Searching in dir: {}", options.path.clone()).magenta()
        );
    }
    let mut output = Output::default();
    if let Ok(entries_vec) = fs::read_dir(options.path.clone()) {
        for may_be_entry in entries_vec {
            if let Ok(entry) = may_be_entry {
                let p: std::path::PathBuf = entry.path();
                let file_type = entry.file_type().unwrap();
                if file_type.is_dir() {
                    output.update(search_folder(
                        options.clone_but_change_path(p.as_os_str().to_str().unwrap().to_string()),
                    ));
                } else if file_type.is_file() {
                    let extension: &str = match p.as_path().extension() {
                        Some(ext) => ext.to_str().unwrap(),
                        None => "",
                    };
                    if HANDLED_EXTENSIONS.contains(&extension) {
                        output.update(search_file(options.clone_but_change_path(format!(
                            "{}\\{}",
                            options.path.clone(),
                            entry.path().file_name().unwrap().to_str().unwrap()
                        ))));
                    } else {
                        if !options.silent {
                            println!(
                                "{}",
                                format!(
                                    "Skipped file koz bad extension: '{}: {}'",
                                    entry.path().file_name().unwrap().to_str().unwrap(),
                                    extension
                                )
                                .red()
                            )
                        }
                    }
                }
            }
        }
    }

    output
}

fn search_file(options: Options) -> Output {
    let mut file = File::new(options.path.clone());
    if !options.silent {
        println!(
            "{}",
            format!("Searching in file: {}", options.path.clone()).cyan()
        );
    }

    // Read the file and update the file var according to it
    match fs::read_to_string(options.path.clone()) {
        Ok(content) => {
            if options.find {
                let list: Vec<(usize, &str)> =
                    content.match_indices(&options.str_to_find).collect();
                let number_of_occurences = list.len();
                // let list_of_indexes: Vec<usize> =
                //     list.into_iter().map(|occurence| occurence.0).collect();
                file.times = number_of_occurences as i32
            }
        }
        Err(e) => {
            let msg = format!(
                "Got an error reading the file: {}\nError: {}",
                options.path.clone(),
                e
            );
            println!("{}", msg.red())
        }
    }

    Output::from(file)
}

fn search(options: Options) -> Output {
    let mut output: Output = Output::default();

    output.update(search_folder(options));

    output
}

fn main() {
    let mut options = Options::default();

    let description = format!(
        "A simple app to search for specific string in files of a given directory, in a recursive way ofc."
    );

    {
        let mut ap = ArgumentParser::new();

        ap.set_description(&description);
        ap.refer(&mut options.path).add_option(
            &["-p", "--path"],
            Store,
            "Modify the searched path",
        );
        ap.refer(&mut options.silent).add_option(
            &["-s", "--silent"],
            StoreTrue,
            "Mutes the search",
        );
        ap.refer(&mut options.str_to_find).add_option(
            &["-f", "--find"],
            Store,
            "The string you are looking for",
        );
        // Maybe someday add checkup for corrupted files
        ap.parse_args_or_exit();
    }

    options.update();
    println!("{}", options.show());

    if !std::path::Path::new(&options.path).exists() {
        let msg = "Please input a correct path".red();
        println!("{}", msg);
        std::process::exit(1)
    }

    println!("{}", search(options.clone()).display(options));
}
