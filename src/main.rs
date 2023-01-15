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

#[derive(PartialEq)]
enum MyOption<T> {
    None,
    Unset,
    Some(T),
}

struct Editorconfig {
    indent_style: MyOption<IndentStyle>,
    indent_size: MyOption<i8>,
    tab_width: MyOption<i8>,
    end_of_line: MyOption<EndOfLine>,
    charset: MyOption<Charset>,
    trim_trailing_whitespace: MyOption<bool>,
    insert_final_newline: MyOption<bool>,
}

impl Editorconfig {
    pub fn new() -> Editorconfig {
        Editorconfig {
            indent_size: MyOption::None,
            indent_style: MyOption::None,
            tab_width: MyOption::None,
            end_of_line: MyOption::None,
            charset: MyOption::None,
            trim_trailing_whitespace: MyOption::None,
            insert_final_newline: MyOption::None,
        }
    }

    pub fn merge(&mut self, editorconfig: Editorconfig) {
        if self.indent_style == MyOption::None
            && (editorconfig.indent_style != MyOption::None
                || editorconfig.indent_style == MyOption::Unset)
        {
            self.indent_style = editorconfig.indent_style
        }

        if self.indent_size == MyOption::None
            && (editorconfig.indent_size != MyOption::None
                || editorconfig.indent_size == MyOption::Unset)
        {
            self.indent_size = editorconfig.indent_size
        }

        if self.tab_width == MyOption::None
            && (editorconfig.tab_width != MyOption::None
                || editorconfig.tab_width == MyOption::Unset)
        {
            self.tab_width = editorconfig.tab_width
        }

        if self.end_of_line == MyOption::None
            && (editorconfig.end_of_line != MyOption::None
                || editorconfig.end_of_line == MyOption::Unset)
        {
            self.end_of_line = editorconfig.end_of_line
        }

        if self.charset == MyOption::None
            && (editorconfig.charset != MyOption::None || editorconfig.charset == MyOption::Unset)
        {
            self.charset = editorconfig.charset
        }

        if self.trim_trailing_whitespace == MyOption::None
            && (editorconfig.trim_trailing_whitespace != MyOption::None
                || editorconfig.trim_trailing_whitespace == MyOption::Unset)
        {
            self.trim_trailing_whitespace = editorconfig.trim_trailing_whitespace
        }

        if self.insert_final_newline == MyOption::None
            && (editorconfig.insert_final_newline != MyOption::None
                || editorconfig.insert_final_newline == MyOption::Unset)
        {
            self.insert_final_newline = editorconfig.insert_final_newline
        }
    }

    pub fn print(self) {
        self.print_indent_style();
        self.print_indent_size();
        self.print_tab_width();
        self.print_trim_trailing_whitespace();
        self.print_insert_final_newline();
        self.print_end_of_line();
        self.print_charset();
    }

    fn print_indent_style(&self) {
        if let MyOption::Some(indent_style) = &self.indent_style {
            match indent_style {
                IndentStyle::Tab => println!("indent_style=tab"),
                IndentStyle::Space => println!("indent_style=space"),
            }
        }
    }

    fn print_indent_size(&self) {
        if let MyOption::Some(indent_size) = self.indent_size {
            println!("indent_size={}", indent_size);
        }
    }

    fn print_tab_width(&self) {
        if let MyOption::Some(tab_width) = self.tab_width {
            println!("tab_width={}", tab_width);
        }
    }

    fn print_trim_trailing_whitespace(&self) {
        if let MyOption::Some(trim_trailing_whitespace) = self.trim_trailing_whitespace {
            println!("trim_trailing_whitespace={}", trim_trailing_whitespace);
        }
    }

    fn print_insert_final_newline(&self) {
        if let MyOption::Some(insert_final_newline) = self.insert_final_newline {
            println!("insert_final_newline={}", insert_final_newline);
        }
    }

    fn print_end_of_line(&self) {
        if let MyOption::Some(end_of_line) = &self.end_of_line {
            match end_of_line {
                EndOfLine::Lf => println!("end_of_line=lf"),
                EndOfLine::Cr => println!("end_of_line=cr"),
                EndOfLine::Crlf => println!("end_of_line=crlf"),
            }
        }
    }

    fn print_charset(&self) {
        if let MyOption::Some(charset) = &self.charset {
            match charset {
                Charset::Latin1 => println!("charset=latin1"),
                Charset::Utf8 => println!("charset=utf-8"),
                Charset::Utf8Bom => println!("charset=utf-8-bom"),
                Charset::Utf16Be => println!("charset=uft-16-be"),
                Charset::Utf16Le => println!("charset=utf-16-le"),
            }
        }
    }
}

#[derive(PartialEq)]
enum IndentStyle {
    Space,
    Tab,
}

#[derive(PartialEq)]
enum EndOfLine {
    Lf,
    Cr,
    Crlf,
}

#[derive(PartialEq)]
enum Charset {
    Latin1,
    Utf8,
    Utf8Bom,
    Utf16Be,
    Utf16Le,
}

type Definitions = HashMap<PathBuf, Editorconfig>;

fn main() {
    let args = Args::parse();
    let paths = make_absolute_paths(args.filepaths);
    let definitions = get_definitions(paths);

    // Testing output
    for definition in definitions {
        println!("file: {:?}", definition.0);
        definition.1.print();
    }
}

fn make_absolute_paths(filepaths: Vec<PathBuf>) -> Vec<PathBuf> {
    filepaths
        .into_iter()
        .filter_map(|f| fs::canonicalize(f).ok())
        .collect()
}

fn get_editorconfig_for_file(filepath: &Path) -> Editorconfig {
    let mut editorconfig: Editorconfig = Editorconfig::new();
    let mut path = filepath.to_path_buf();

    while path.pop() {
        if let Some(file) = fs::read_dir(&path)
            .expect("read_dir call failed")
            .flatten()
            .find(|x| x.path().ends_with(".editorconfig"))
        {
            let ini = Ini::load_from_file(file.path()).unwrap();
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
                            editorconfig.merge(Editorconfig {
                                indent_style: get_indent_style(&ini, section),
                                indent_size: get_indent_size(&ini, section),
                                tab_width: get_tab_width(&ini, section),
                                charset: get_charset(&ini, section),
                                end_of_line: get_end_of_line(&ini, section),
                                trim_trailing_whitespace: get_trim_trailing_whitespace(
                                    &ini, section,
                                ),
                                insert_final_newline: get_insert_final_newline(&ini, section),
                            });
                        };
                    }
                }
            });

            if is_root {
                break;
            }
        }
    }

    editorconfig
}

fn get_definitions(paths: Vec<PathBuf>) -> Definitions {
    paths
        .into_iter()
        .fold(HashMap::new(), |mut map: Definitions, path| {
            map.insert(path.clone(), get_editorconfig_for_file(&path));
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

fn get_indent_style(ini: &Ini, section: &str) -> MyOption<IndentStyle> {
    let indent_style_option = ini.section(Some(section)).unwrap().get("indent_style");

    match indent_style_option {
        Some(indent_style) => match indent_style {
            "space" => MyOption::Some(IndentStyle::Space),
            "tab" => MyOption::Some(IndentStyle::Tab),
            "unset" => MyOption::Unset,
            _ => MyOption::None,
        },
        None => MyOption::None,
    }
}

fn get_indent_size(ini: &Ini, section: &str) -> MyOption<i8> {
    let indent_size_option = ini.section(Some(section)).unwrap().get("indent_size");

    match indent_size_option {
        Some(indent_size) => match indent_size {
            "unset" => MyOption::Unset,
            x => x.parse::<i8>().map_or(MyOption::None, MyOption::Some),
        },
        None => MyOption::None,
    }
}

fn get_tab_width(ini: &Ini, section: &str) -> MyOption<i8> {
    let tab_width_option = ini.section(Some(section)).unwrap().get("tab_width");

    match tab_width_option {
        Some(tab_width) => match tab_width {
            "unset" => MyOption::Unset,
            x => x.parse::<i8>().map_or(MyOption::None, MyOption::Some),
        },
        None => MyOption::None,
    }
}

fn get_trim_trailing_whitespace(ini: &Ini, section: &str) -> MyOption<bool> {
    let trim_trailing_whitespace_option = ini
        .section(Some(section))
        .unwrap()
        .get("trim_trailing_whitespace");

    match trim_trailing_whitespace_option {
        Some(trim_trailing_whitespace) => match trim_trailing_whitespace {
            "unset" => MyOption::Unset,
            x => x.parse::<bool>().map_or(MyOption::None, MyOption::Some),
        },
        None => MyOption::None,
    }
}

fn get_insert_final_newline(ini: &Ini, section: &str) -> MyOption<bool> {
    let insert_final_newline_option = ini
        .section(Some(section))
        .unwrap()
        .get("insert_final_newline");

    match insert_final_newline_option {
        Some(insert_final_newline) => match insert_final_newline {
            "unset" => MyOption::Unset,
            x => x.parse::<bool>().map_or(MyOption::None, MyOption::Some),
        },
        None => MyOption::None,
    }
}

fn get_end_of_line(ini: &Ini, section: &str) -> MyOption<EndOfLine> {
    let end_of_line_option = ini.section(Some(section)).unwrap().get("end_of_line");

    match end_of_line_option {
        Some(end_of_line) => match end_of_line {
            "lf" => MyOption::Some(EndOfLine::Lf),
            "cr" => MyOption::Some(EndOfLine::Cr),
            "crlf" => MyOption::Some(EndOfLine::Crlf),
            "unset" => MyOption::Unset,
            _ => MyOption::None,
        },
        None => MyOption::None,
    }
}

fn get_charset(ini: &Ini, section: &str) -> MyOption<Charset> {
    let charset_option = ini.section(Some(section)).unwrap().get("charset");

    match charset_option {
        Some(charset) => match charset {
            "latin1" => MyOption::Some(Charset::Latin1),
            "utf-8" => MyOption::Some(Charset::Utf8),
            "utf-8-bom" => MyOption::Some(Charset::Utf8Bom),
            "utf-16-be" => MyOption::Some(Charset::Utf16Be),
            "utf-16-le" => MyOption::Some(Charset::Utf16Le),
            "unset" => MyOption::Unset,
            _ => MyOption::None,
        },
        None => MyOption::None,
    }
}
