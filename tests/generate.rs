use glob::glob;
use quicli::prelude::*;

#[test]
fn generate_test_cases() -> CliResult {
    let _ = env_logger::builder().is_test(true).filter(None, log::LevelFilter::Debug).try_init();
    let target_dir = std::path::PathBuf::from("./tests/test-package/examples");
    create_dir(&target_dir)?;

    for filename in glob("tests/fixtures/*.yml")? {
        let filename = filename?;
        let content = read_file(&filename)?;
        let openapi: openapiv3::OpenAPI = serde_yaml::from_str(&content)?;
        let mut code = openapi_alors::generate::rust::types(&openapi)?;
        code.push_str("fn main() {}");

        let name = filename
            .file_stem()
            .ok_or_else(|| format_err!("file `{}` has no extension", filename.display()))?;

        write_to_file(target_dir.join(name).with_extension("rs"), &code)?;
    }

    let fmt = std::process::Command::new("cargo")
        .arg("fmt")
        .current_dir("./tests/test-package/")
        .status()?;

    if !fmt.success() {
        Err(format_err!("cargo fmt failed"))?;
    }

    let fmt = std::process::Command::new("cargo")
        .arg("check")
        .arg("--all")
        .arg("--all-targets")
        .current_dir("./tests/test-package/")
        .status()?;

    if !fmt.success() {
        Err(format_err!("cargo check failed"))?;
    }

    Ok(())
}
