use lightningcss::{
    stylesheet::{MinifyOptions, ParserOptions, PrinterOptions, StyleSheet},
    targets::Browsers,
};
use std::{fs, path::Path, path::PathBuf};
use xshell::{cmd, Shell};

use crate::{config::Config, Error, Reportable};

pub fn run(config: &Config) -> Result<(), Reportable> {
    let style = &config.style;
    let scss_file = &style.file;

    log::debug!("Style found: {scss_file:?}");
    let scss_file = Path::new(scss_file);
    if !scss_file.exists() || !scss_file.is_file() {
        return Err(Reportable::not_a_file("expected an scss file", scss_file));
    }
    let css_file = compile_scss(scss_file, config.release)
        .map_err(|e| e.file_context("compile scss", scss_file))?;

    let browsers = browser_lists(&style.browserquery)
        .map_err(|e| e.config_context("leptos.style.browserquery"))?;

    process_css(&css_file, browsers, config.release)
        .map_err(|e| e.file_context("process css", scss_file))?;

    Ok(())
}

fn compile_scss(file: &Path, release: bool) -> Result<PathBuf, Error> {
    let dest = format!("target/site/pkg/app.css");
    let sourcemap = release.then(|| "--no-source-map");

    let sh = Shell::new()?;
    cmd!(sh, "sass {file} {dest} {sourcemap...}").run()?;
    Ok(PathBuf::from(dest))
}

fn browser_lists(query: &str) -> Result<Option<Browsers>, Error> {
    Browsers::from_browserslist([query]).map_err(|e| Error::BrowserListError(e.to_string()))
}

fn process_css(file: &Path, browsers: Option<Browsers>, release: bool) -> Result<(), Error> {
    let css = fs::read_to_string(&file)?;
    let mut style = StyleSheet::parse(&css, ParserOptions::default())?;

    if release {
        style.minify(MinifyOptions::default())?;
    }

    let mut options = PrinterOptions::default();
    options.targets = browsers;

    if release {
        options.minify = true;
    }
    let style_output = style.to_css(options)?;

    fs::write(&file, style_output.code.as_bytes())?;
    Ok(())
}