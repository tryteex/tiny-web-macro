extern crate proc_macro;

use proc_macro::{Span, TokenStream};
use quote::quote;
use std::{
    process::Command,
    collections::{hash_map::Entry, HashMap},
    env,
    fs::{read_dir, read_to_string},
    str::FromStr,
};
use syn::{parse_macro_input, Error};

/// Recursively build engine system
#[proc_macro]
pub fn addfn(_: TokenStream) -> TokenStream {
    let dir = match env::var_os("CARGO_MANIFEST_DIR") {
        Some(d) => match d.to_str() {
            Some(s) => s.to_owned(),
            None => return error("CARGO_MANIFEST_DIR contains non-printable characters"),
        },
        None => return error("Can't fetch the environment variable CARGO_MANIFEST_DIR"),
    };
    let list = load_files(&dir).unwrap_or_default();

    let mut vec = Vec::new();
    vec.push("let mut app: std::collections::BTreeMap<i64, std::collections::BTreeMap<i64, std::collections::BTreeMap<i64, tiny_web::sys::action::Act>>> = std::collections::BTreeMap::new();"
            .to_string(),
    );
    for (key, v) in list {
        vec.push(format!(
            "let mut {}: std::collections::BTreeMap<i64, std::collections::BTreeMap<i64, tiny_web::sys::action::Act>> = std::collections::BTreeMap::new();",
            key
        ));
        for file in v {
            let func = get_func(&dir, &key, &file);
            vec.push(format!(
                "let mut {}_{}: std::collections::BTreeMap<i64, tiny_web::sys::action::Act> = std::collections::BTreeMap::new();",
                key, file
            ));
            for f in func {
                vec.push(format!(
                    "{0}_{1}.insert({2}_i64, |action| Box::pin(app::{0}::{1}::{3}(action)));",
                    key,
                    file,
                    fnv1a_64_impl(&f),
                    f
                ));
            }
            vec.push(format!(
                "{0}.insert({1}_i64, {0}_{2});",
                key,
                fnv1a_64_impl(&file),
                file
            ));
        }
        vec.push(format!("app.insert({}_i64, {});", fnv1a_64_impl(&key), key));
    }
    vec.push("\nreturn app;".to_owned());

    TokenStream::from_str(&vec.join("\n")).unwrap()
}

/// Gets functions list from directory
///
/// Each function have to start `pub async fn ` and finish `( this : &mut Action ) -> Answer {`.
fn get_func(dir: &str, key: &str, file: &str) -> Vec<String> {
    let mut vec = Vec::new();
    let file = format!("{}/src/app/{}/{}.rs", dir, key, file);
    if let Ok(str) = read_to_string(file) {
        let mut str = str
            .replace('(', " ( ")
            .replace(')', " ) ")
            .replace(':', " : ")
            .replace("->", " -> ")
            .replace('{', " { ");
        loop {
            if str.contains("  ") {
                str = str.replace("  ", " ");
                continue;
            }
            break;
        }
        for line in str.lines() {
            if let Some(i) = line.find("pub async fn ") {
                if let Some(j) = line[i + 13..].find(" ( ") {
                    if line[i + 13 + j..].contains(" : &mut Action ) -> Answer {") {
                        let name = &line[i + 13..i + 13 + j];
                        if name.chars().all(|c| c.is_ascii_lowercase() || c == '_') {
                            vec.push(line[i + 13..i + 13 + j].to_owned());
                        }
                    }
                }
            }
        }
    }
    vec.shrink_to_fit();
    vec
}

/// Recursively links all files with `.rs` extension from `./src/app/*` directory.
#[proc_macro]
pub fn addmod(_: TokenStream) -> TokenStream {
    // Get project dir
    let dir = match env::var_os("CARGO_MANIFEST_DIR") {
        Some(d) => match d.to_str() {
            Some(s) => s.to_owned(),
            None => return error("CARGO_MANIFEST_DIR contains non-printable characters"),
        },
        None => return error("Can't fetch the environment variable CARGO_MANIFEST_DIR"),
    };
    // load all files
    let list = match load_files(&dir) {
        Ok(l) => l,
        Err(e) => return error(&e),
    };
    // Forms an answer
    let mut vec = Vec::new();
    for (key, v) in list {
        vec.push(format!("pub mod {} {{", check_name(key)));
        for f in v {
            vec.push(format!("    pub mod {};", check_name(f)));
        }
        vec.push("}".to_owned());
    }
    TokenStream::from_str(&vec.join("\n")).unwrap()
}

/// If the name contains the symbol "-", it replaces it with "_"
fn check_name(text: String) -> String {
    if text.contains('-') {
        return text.replace('-', "_");
    }
    text
}

/// Load all file names with `.rs` extension from `./src/app/*` directory
fn load_files(dir: &str) -> Result<HashMap<String, Vec<String>>, String> {
    let src = format!("{}/src/app", dir);
    let mut list: HashMap<String, Vec<String>> = HashMap::new();
    // Reads dir from first level
    match read_dir(&src) {
        Ok(dir) => {
            for entry in dir.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                if let Some(name) = path.file_name() {
                    let dir_name = match name.to_str() {
                        Some(n) => n,
                        None => continue,
                    };
                    // Reads dir from second level
                    let dir = match read_dir(format!("{}/{}", &src, dir_name)) {
                        Ok(d) => d,
                        Err(_) => continue,
                    };
                    for entry in dir.flatten() {
                        let path = entry.path();
                        if !path.is_file() {
                            continue;
                        }
                        let file_name = match path.file_name() {
                            Some(name) => match name.to_str() {
                                Some(file_name) => file_name,
                                None => continue,
                            },
                            None => continue,
                        };
                        // Checks extension
                        if file_name.len() > 3 && file_name.ends_with(".rs") {
                            let file_name = file_name[..file_name.len() - 3].to_owned();
                            match list.entry(dir_name.to_owned()) {
                                Entry::Occupied(mut o) => {
                                    let vec = o.get_mut();
                                    vec.push(file_name);
                                    vec.shrink_to_fit();
                                }
                                Entry::Vacant(v) => {
                                    let vec = vec![file_name];
                                    v.insert(vec);
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(e) => return Err(format!("{}. File name: {}", e, src)),
    };
    list.shrink_to_fit();
    Ok(list)
}

/// Returns error text
fn error(text: &str) -> TokenStream {
    TokenStream::from(Error::new(Span::call_site().into(), text).to_compile_error())
}

/// fnv1a_64 hash function
///
/// # Parameters
///
/// * `text: &str` - Origin string.
///
/// # Return
///
/// i64 hash
#[inline]
fn fnv1a_64_impl(text: &str) -> i64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    let prime: u64 = 0x100000001b3;

    for c in text.bytes() {
        hash ^= u64::from(c);
        hash = hash.wrapping_mul(prime);
    }
    unsafe { *(&hash as *const u64 as *const i64) }
}

/// fnv1a_64 hash function
/// Apply only for static &str
#[proc_macro]
pub fn fnv1a_64(params: TokenStream) -> TokenStream {
    let text = parse_macro_input!(params as syn::LitStr).value();
    let result = fnv1a_64_impl(&text);
    let result_token_stream = quote! { #result };
    result_token_stream.into()
}

/// Return the version of the Rust compiler
#[proc_macro]
pub fn version(_: TokenStream) -> TokenStream {
    let output = match Command::new("rustc").arg("--version").output() {
        Ok(output) => output,
        Err(e) => return error(&format!("Can't execute command `rustc --version`: {}", e)),
    };
    let result = String::from_utf8_lossy(&output.stdout);
    if !result.starts_with("rustc ") {
        return error(&format!("`rustc --version` returns an incorrect version string: {}", result))
    }

    let result_token_stream = quote! { #result };
    result_token_stream.into()
}
