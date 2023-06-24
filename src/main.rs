use std::collections::HashMap;
use std::ffi::CString;

use std::path::Path;
use std::path::PathBuf;

use path_clean::clean;

use clap::Parser;
use ini::Ini;

#[derive(Parser, Debug)]
#[command(about, long_about = None)]
struct Args {
    /// Specify conf filename other than '.editorconfig'
    #[arg(short, default_value_t = String::from(".editorconfig"))]
    f: String,

    /// Specify version (used by devs to test compatibility)
    #[arg(short, default_value_t = String::from(""))]
    b: String,

    /// Files to find configuration for.  Can be a hyphen (-) if you want path(s) to be read from stdin.
    #[arg(name("FILEPATH"))]
    filepaths: Vec<PathBuf>,

    /// Show the version
    #[arg(short, long, default_value_t = false)]
    version: bool,
}

type Definitions = HashMap<PathBuf, FileDefinition>;
type FileDefinition = HashMap<String, String>;

fn merge(fst: &FileDefinition, snd: &FileDefinition) -> FileDefinition {
    let mut file_definition = fst.clone();

    for (key, value) in snd.iter() {
        if !file_definition.contains_key(key) {
            file_definition.insert(key.to_owned(), value.to_owned());
        }
    }

    if file_definition.get("indent_style") == Some(&String::from("tab"))
        && !file_definition.contains_key("indent_size")
    {
        file_definition.insert(String::from("indent_size"), String::from("tab"));
    }

    if file_definition.contains_key("indent_size")
        && file_definition.get("indent_size") != Some(&String::from("tab"))
        && !file_definition.contains_key("tab_width")
    {
        file_definition.insert(
            String::from("tab_width"),
            String::from(file_definition.get("indent_size").unwrap()),
        );
    }

    if file_definition.get("indent_size") == Some(&String::from("tab"))
        && file_definition.contains_key("tab_width")
    {
        file_definition.insert(
            String::from("indent_size"),
            String::from(file_definition.get("tab_width").unwrap()),
        );
    }

    file_definition
}

fn main() {
    let args = Args::parse();

    if args.version {
        println!("editorconfig-core-rust: {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    let paths = make_absolute_paths(args.filepaths);
    let definitions = get_definitions(paths, &args.f);

    // Testing output
    for definition in definitions {
        for (key, value) in definition.1.iter() {
            println!("{}={}", key.to_lowercase(), value);
        }
    }
}

fn make_absolute_paths(filepaths: Vec<PathBuf>) -> Vec<PathBuf> {
    filepaths.into_iter().map(clean).collect()
}

fn path_contains_editoconfig(path: &Path, editorconfig_file: &str) -> bool {
    std::path::Path::new(&path.join(editorconfig_file)).exists()
}

fn get_editorconfig_for_file(filepath: &Path, editorconfig_file: &str) -> FileDefinition {
    let mut file_definition = FileDefinition::new();
    let mut path = filepath.to_path_buf();

    while path.pop() {
        // println!(
        //     "path_contains_editoconfig(path, editorconfig_file): {:?} {:?}",
        //     path,
        //     path_contains_editoconfig(&path, editorconfig_file)
        // );
        if path_contains_editoconfig(&path, editorconfig_file) {
            let ini = Ini::load_from_file(&path.join(editorconfig_file)).unwrap();
            let is_root = is_root(&ini);

            let sections = ini.sections().map(|x| x.unwrap_or(""));

            sections.into_iter().for_each(|section| {
                if !section.is_empty() {
                    let pattern = CString::new(section).unwrap();
                    let p = CString::new(filepath.to_str().unwrap()).unwrap();

                    unsafe {
                        let matches = fnmatch_sys::fnmatch(
                            pattern.as_ptr(),
                            p.as_ptr(),
                            fnmatch_sys::FNM_NOESCAPE,
                        ) == 0
                            || section == filepath.file_name().unwrap();

                        if matches {
                            file_definition =
                                merge(&file_definition, &get_section_definition(&ini, section));
                        };
                    }
                }
            });

            if is_root {
                break;
            }
        }
    }

    file_definition
}

fn get_definitions(paths: Vec<PathBuf>, editorconfig_file: &str) -> Definitions {
    paths
        .into_iter()
        .fold(HashMap::new(), |mut map: Definitions, path| {
            map.insert(
                path.clone(),
                get_editorconfig_for_file(&path, editorconfig_file),
            );
            map
        })
}

fn get_section_definition(ini: &Ini, section: &str) -> FileDefinition {
    ini.section(Some(section)).unwrap().iter().fold(
        FileDefinition::new(),
        |mut map: FileDefinition, (key, value)| {
            map.insert(key.to_owned(), value.to_owned());
            map
        },
    )
}

fn is_root(ini: &Ini) -> bool {
    if let Some(general_section) = ini.section(None::<String>) {
        if let Some(is_root) = general_section.get("root") {
            if is_root == "true" {
                return true;
            }
        }
    }

    false
}
