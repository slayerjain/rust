// Copyright 2016 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fs::{self, File};
use std::io::prelude::*;
use std::path::Path;
use std::process::Command;

use build::{Build, Compiler, Mode};
use build::util::{up_to_date, cp_r};

pub fn rustbook(build: &Build, stage: u32, host: &str, name: &str, out: &Path) {
    t!(fs::create_dir_all(out));

    let out = out.join(name);
    let compiler = Compiler::new(stage, host);
    let src = build.src.join("src/doc").join(name);
    let index = out.join("index.html");
    let rustbook = build.tool(&compiler, "rustbook");
    if up_to_date(&src, &index) && up_to_date(&rustbook, &index) {
        return
    }
    println!("Rustbook stage{} ({}) - {}", stage, host, name);
    let _ = fs::remove_dir_all(&out);
    build.run(build.tool_cmd(&compiler, "rustbook")
                   .arg("build")
                   .arg(&src)
                   .arg(out));
}

pub fn standalone(build: &Build, stage: u32, host: &str, out: &Path) {
    println!("Documenting stage{} standalone ({})", stage, host);
    t!(fs::create_dir_all(out));

    let compiler = Compiler::new(stage, host);

    let favicon = build.src.join("src/doc/favicon.inc");
    let footer = build.src.join("src/doc/footer.inc");
    let full_toc = build.src.join("src/doc/full-toc.inc");
    t!(fs::copy(build.src.join("src/doc/rust.css"), out.join("rust.css")));

    let version_input = build.src.join("src/doc/version_info.html.template");
    let version_info = out.join("version_info.html");

    if !up_to_date(&version_input, &version_info) {
        let mut info = String::new();
        t!(t!(File::open(&version_input)).read_to_string(&mut info));
        let blank = String::new();
        let short = build.short_ver_hash.as_ref().unwrap_or(&blank);
        let hash = build.ver_hash.as_ref().unwrap_or(&blank);
        let info = info.replace("VERSION", &build.release)
                       .replace("SHORT_HASH", short)
                       .replace("STAMP", hash);
        t!(t!(File::create(&version_info)).write_all(info.as_bytes()));
    }

    for file in t!(fs::read_dir(build.src.join("src/doc"))) {
        let file = t!(file);
        let path = file.path();
        let filename = path.file_name().unwrap().to_str().unwrap();
        if !filename.ends_with(".md") || filename == "README.md" {
            continue
        }

        let html = out.join(filename).with_extension("html");
        let rustdoc = build.rustdoc(&compiler);
        if up_to_date(&path, &html) &&
           up_to_date(&footer, &html) &&
           up_to_date(&favicon, &html) &&
           up_to_date(&full_toc, &html) &&
           up_to_date(&version_info, &html) &&
           up_to_date(&rustdoc, &html) {
            continue
        }

        let mut cmd = Command::new(&rustdoc);
        build.add_rustc_lib_path(&compiler, &mut cmd);
        cmd.arg("--html-after-content").arg(&footer)
           .arg("--html-before-content").arg(&version_info)
           .arg("--html-in-header").arg(&favicon)
           .arg("--markdown-playground-url")
           .arg("https://play.rust-lang.org/")
           .arg("-o").arg(out)
           .arg(&path);

        if filename == "reference.md" {
           cmd.arg("--html-in-header").arg(&full_toc);
        }

        if filename == "not_found.md" {
            cmd.arg("--markdown-no-toc")
               .arg("--markdown-css")
               .arg("https://doc.rust-lang.org/rust.css");
        } else {
            cmd.arg("--markdown-css").arg("rust.css");
        }
        build.run(&mut cmd);
    }
}

pub fn std(build: &Build, stage: u32, host: &str, out: &Path) {
    println!("Documenting stage{} std ({})", stage, host);
    let compiler = Compiler::new(stage, host);
    let out_dir = build.stage_out(&compiler, Mode::Libstd)
                       .join(host).join("doc");
    let rustdoc = build.rustdoc(&compiler);

    build.clear_if_dirty(&out_dir, &rustdoc);

    let mut cargo = build.cargo(&compiler, Mode::Libstd, host, "doc");
    cargo.arg("--manifest-path")
         .arg(build.src.join("src/rustc/std_shim/Cargo.toml"))
         .arg("--features").arg(build.std_features());
    build.run(&mut cargo);
    cp_r(&out_dir, out)
}

pub fn rustc(build: &Build, stage: u32, host: &str, out: &Path) {
    println!("Documenting stage{} compiler ({})", stage, host);
    let compiler = Compiler::new(stage, host);
    let out_dir = build.stage_out(&compiler, Mode::Librustc)
                       .join(host).join("doc");
    let rustdoc = build.rustdoc(&compiler);
    if !up_to_date(&rustdoc, &out_dir.join("rustc/index.html")) {
        t!(fs::remove_dir_all(&out_dir));
    }
    let mut cargo = build.cargo(&compiler, Mode::Librustc, host, "doc");
    cargo.arg("--manifest-path")
         .arg(build.src.join("src/rustc/Cargo.toml"))
         .arg("--features").arg(build.rustc_features());
    build.run(&mut cargo);
    cp_r(&out_dir, out)
}

pub fn error_index(build: &Build, stage: u32, host: &str, out: &Path) {
    println!("Documenting stage{} error index ({})", stage, host);
    let compiler = Compiler::new(stage, host);
    let mut index = build.tool_cmd(&compiler, "error_index_generator");
    index.arg("html");
    index.arg(out.join("error-index.html"));

    // FIXME: shouldn't have to pass this env var
    index.env("CFG_BUILD", &build.config.build);

    build.run(&mut index);
}
