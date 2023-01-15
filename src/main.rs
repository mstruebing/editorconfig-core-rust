use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use clap::Parser;
use ini::Ini;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Specify conf filename other than '.editorconfig'
    #[arg(short, default_value_t = String::from(".editorconfig"))]
    f: String,

    /// Specify version (used by devs to test compatibility)
    #[arg(short, default_value_t = String::from(""))]
    b: String,

    /// Files to find configuration for.  Can be a hyphen (-) if you want path(s) to be read from stdin.
    #[arg(name("FILEPATH"), required(true))]
    filepaths: Vec<PathBuf>,
}

// TODO: Enhance
struct Editorconfig {
    path: PathBuf,
    raw: Ini,
}

type Definitions = HashMap<PathBuf, Vec<Editorconfig>>;

fn main() {
    let args = Args::parse();
    let paths = make_absolute_paths(args.filepaths);
    let definitions = get_definitions(paths);

    // Testing output
    for definition in definitions {
        println!("file: {:?}", definition.0);

        definition
            .1
            .into_iter()
            .for_each(|editorconfig| println!("{}: ", editorconfig.path.display()))
    }
}

fn make_absolute_paths(filepaths: Vec<PathBuf>) -> Vec<PathBuf> {
    filepaths
        .into_iter()
        .filter_map(|f| fs::canonicalize(f).ok())
        .collect()
}

fn get_editorconfigs_for_file(filepath: &Path) -> Vec<Editorconfig> {
    let mut editorconfigs: Vec<Editorconfig> = Vec::new();
    let mut path = filepath.to_path_buf();

    while path.pop() {
        if let Some(editorconfig) = fs::read_dir(&path)
            .expect("read_dir call failed")
            .flatten()
            .find(|x| x.path().ends_with(".editorconfig"))
        {
            let ini = Ini::load_from_file(editorconfig.path()).unwrap();
            let is_root = is_root(&ini);

            editorconfigs.insert(
                0,
                Editorconfig {
                    path: editorconfig.path(),
                    raw: ini,
                },
            );

            if is_root {
                break;
            }
        }
    }

    editorconfigs
}

fn get_definitions(paths: Vec<PathBuf>) -> Definitions {
    paths
        .into_iter()
        .fold(HashMap::new(), |mut map: Definitions, path| {
            map.insert(path.clone(), get_editorconfigs_for_file(&path));
            map
        })
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
