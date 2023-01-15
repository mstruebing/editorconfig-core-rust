use std::collections::HashMap;
use std::ffi::CString;
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

#[derive(Debug)]
struct Editorconfig {
    indent_style: Option<IndentStyle>,
    indent_size: Option<i8>,
    tab_width: Option<i8>,
    end_of_line: Option<EndOfLine>,
    charset: Option<Charset>,
    trim_trailing_whitespace: Option<bool>,
    insert_final_newline: Option<bool>,
}

#[derive(Debug)]
enum IndentStyle {
    Space,
    Tab,
}

#[derive(Debug)]
enum EndOfLine {
    Lf,
    Cr,
    Crlf,
}

#[derive(Debug)]
enum Charset {
    Latin1,
    Utf8,
    Utf8Bom,
    Utf16Be,
    Utf16Le,
}

type Definitions = HashMap<PathBuf, Vec<Editorconfig>>;

fn main() {
    let args = Args::parse();
    let paths = make_absolute_paths(args.filepaths);
    let definitions = get_definitions(paths);

    // Testing output
    for definition in definitions {
        println!("file: {:?}", definition.0);
        println!("file: {:?}", definition.1);
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
                            editorconfigs.insert(
                                0,
                                Editorconfig {
                                    indent_style: get_indent_style(&ini, section),
                                    indent_size: get_indent_size(&ini, section),
                                    tab_width: get_tab_width(&ini, section),
                                    charset: get_charset(&ini, section),
                                    end_of_line: get_end_of_line(&ini, section),
                                    trim_trailing_whitespace: get_trim_trailing_whitespace(
                                        &ini, section,
                                    ),
                                    insert_final_newline: get_insert_final_newline(&ini, section),
                                },
                            )
                        }
                    }
                }
            });

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

fn get_indent_style(ini: &Ini, section: &str) -> Option<IndentStyle> {
    let indent_style = ini
        .section(Some(section))
        .unwrap()
        .get("indent_style")
        .and_then(|x| match x {
            "space" => Some(IndentStyle::Space),
            "tab" => Some(IndentStyle::Tab),
            _ => None,
        });

    indent_style
}

fn get_indent_size(ini: &Ini, section: &str) -> Option<i8> {
    let indent_size = ini
        .section(Some(section))
        .unwrap()
        .get("indent_size")
        .and_then(|x| x.parse::<i8>().ok());

    indent_size
}

fn get_tab_width(ini: &Ini, section: &str) -> Option<i8> {
    let tab_width = ini
        .section(Some(section))
        .unwrap()
        .get("tab_width")
        .and_then(|x| x.parse::<i8>().ok());

    tab_width
}

fn get_trim_trailing_whitespace(ini: &Ini, section: &str) -> Option<bool> {
    let trim_trailing_whitespace = ini
        .section(Some(section))
        .unwrap()
        .get("trim_trailing_whitespace")
        .and_then(|x| x.parse::<bool>().ok());

    trim_trailing_whitespace
}

fn get_insert_final_newline(ini: &Ini, section: &str) -> Option<bool> {
    let insert_final_newline = ini
        .section(Some(section))
        .unwrap()
        .get("insert_final_newline")
        .and_then(|x| x.parse::<bool>().ok());

    insert_final_newline
}

fn get_end_of_line(ini: &Ini, section: &str) -> Option<EndOfLine> {
    let end_of_line = ini
        .section(Some(section))
        .unwrap()
        .get("end_of_line")
        .and_then(|x| match x {
            "lf" => Some(EndOfLine::Lf),
            "cr" => Some(EndOfLine::Cr),
            "crlf" => Some(EndOfLine::Crlf),
            _ => None,
        });

    end_of_line
}

fn get_charset(ini: &Ini, section: &str) -> Option<Charset> {
    let charset = ini
        .section(Some(section))
        .unwrap()
        .get("charset")
        .and_then(|x| match x {
            "latin1" => Some(Charset::Latin1),
            "utf-8" => Some(Charset::Utf8),
            "utf-8-bom" => Some(Charset::Utf8Bom),
            "utf-16-be" => Some(Charset::Utf16Be),
            "utf-16-le" => Some(Charset::Utf16Le),
            _ => None,
        });

    charset
}
