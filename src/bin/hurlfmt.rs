/*
 * hurl (https://hurl.dev)
 * Copyright (C) 2020 Orange
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *          http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */
extern crate clap;

use std::fs;
use std::io::{self, Read};
use std::io::Write;
use std::path::Path;
use std::process;

use atty::Stream;

use hurl::core::common::FormatError;
use hurl::format::html;
use hurl::format::text;
use hurl::linter::core::Lintable;
use hurl::parser;

fn main() {
    //    // Do we have a git hash?
    //    // (Yes, if ripgrep was built on a machine with `git` installed.)
    //    let hash = match revision_hash.or(option_env!("HURL_BUILD_GIT_HASH")) {
    //             None => String::new(),
    //             Some(githash) => format!(" (rev {})", githash),
    //    };

    let app = clap::App::new("hurlfmt")
        // .author(clap::crate_authors!())
        .version(clap::crate_version!())
        .about("Format hurl FILE or standard input")
        .arg(
            clap::Arg::with_name("INPUT")
                .help("Sets the input file to use")
                .required(false)
                .index(1),
        )
        .arg(
            clap::Arg::with_name("color")
                .long("color")
                .conflicts_with("no-color")
                .help("Colorize Output"),
        )
        .arg(
            clap::Arg::with_name("no_color")
                .long("no-color")
                .conflicts_with("color")
                .help("Do not colorize Output"),
        )
        .arg(
            clap::Arg::with_name("no_format")
                .long("no-format")
                .help("Do not format Output"),
        )
        .arg(
            clap::Arg::with_name("html_output")
                .long("html")
                .conflicts_with("ast_output")
                .help("Output Html"),
        )
        .arg(
            clap::Arg::with_name("standalone")
                .long("standalone")
                .conflicts_with("ast_output")
                .help("Standalone Html"),
        )
        .arg(
            clap::Arg::with_name("html_css_output")
                .long("html-css")
                .conflicts_with("ast_output")
                .help("Output Html"),
        )
        .arg(
            clap::Arg::with_name("ast_output")
                .long("ast")
                .conflicts_with("html_output")
                .help("Output AST"),
        )
        .arg(
            clap::Arg::with_name("check")
                .long("check")
                .conflicts_with("ast_output")
                .conflicts_with("html_output")
                .help("Run in 'check' mode. Exits with 0 if input is\nformatted correctly. Exits with 1 and prints a diff if\nformatting is required"),
        )

        .arg(
            clap::Arg::with_name("in_place")
                .long("in-place")
                .conflicts_with("ast_output")
                .conflicts_with("html_output")
                .conflicts_with("color")
                .help("Modify file in place"),
        );

    let matches = app.clone().get_matches();

    // can you do this check directly with clap
    if matches.is_present("standalone") && !matches.is_present("html_output") {
        eprintln!("use standalone option only with html output");
        std::process::exit(1);
    }

    let output_color = if matches.is_present("color") {
        true
    } else if matches.is_present("no_color") {
        false
    } else {
        atty::is(Stream::Stdout)
    };

    let filename = match matches.value_of("INPUT") {
        None => "-",
        Some("-") => "-",
        Some(v) => v,
    };

    if filename == "-" && atty::is(Stream::Stdin) {
        if app.clone().print_help().is_err() {
            panic!("error during print_help")
        }
        std::process::exit(1);
    } else if filename != "-" && !Path::new(filename).exists() {
        eprintln!("Input file {} does not exit!", filename);
        std::process::exit(1);
    };

    if matches.is_present("in_place") && filename == "-" {
        eprintln!("You can not use inplace with standard input stream!");
        std::process::exit(1);
    };

    let contents = if filename == "-" {
        let mut contents = String::new();
        io::stdin()
            .read_to_string(&mut contents)
            .expect("Something went wrong reading standard input");
        contents
    } else {
        fs::read_to_string(filename).expect("Something went wrong reading the file")
    };

    let lines: Vec<&str> = regex::Regex::new(r"\n|\r\n")
        .unwrap()
        .split(&contents)
        .collect();

    let lines = lines.iter().map(|s| (*s).to_string()).collect();
    match parser::parse_hurl_file(contents.as_str()) {
        Err(e) => {
            //eprintln!("Error {:?}", e);

            let error = hurl::format::error::Error {
                source_info: e.source_info(),
                description: e.description(),
                fixme: e.fixme(),
                lines,
                filename: filename.to_string(),
                warning: true,
                color: output_color,
            };
            eprintln!("{}", error.format());
            process::exit(2);
        }
        Ok(hurl_file) => {
            if matches.is_present("check") {
                for e in hurl_file.errors() {
                    let error = hurl::format::error::Error {
                        source_info: e.source_info(),
                        description: e.description(),
                        fixme: e.fixme(),
                        lines: lines.clone(),
                        filename: filename.to_string(),
                        warning: true,
                        color: output_color,
                    };
                    eprintln!("{}", error.format());
                }
                std::process::exit(1);
            } else if matches.is_present("ast_output") {
                eprintln!("{:#?}", hurl_file);
            } else if matches.is_present("html_output") {
                let standalone = matches.is_present("standalone");
                println!("{}", html::format(hurl_file, standalone));
            } else {
                let hurl_file = if matches.is_present("no_format") {
                    hurl_file
                } else {
                    hurl_file.lint()
                };
                if matches.is_present("in_place") {
                    match fs::File::create(filename) {
                        Ok(mut f) => {
                            let s = text::format(hurl_file, false);
                            f.write_all(s.as_bytes()).unwrap();
                        }
                        Err(_) => eprintln!("Error opening file {} in write mode", filename),
                    };
                } else {
                    print!("{}", text::format(hurl_file, output_color));
                };
            }
        }
    }
}
