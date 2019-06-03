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
        let tokens = openapi_alors::generate::rust::types(&openapi)?;

        let name = filename
            .file_stem()
            .ok_or_else(|| format_err!("file `{}` has no extension", filename.display()))?;

        write_to_file(target_dir.join(name).with_extension("rs"), &tokens.to_string())?;
    }

    let fmt = std::process::Command::new("cargo")
        .arg("fmt")
        .current_dir("./tests/test-package/")
        .status()?;

    if !fmt.success() {
        Err(format_err!("cargo fmt failed"))?;
    }

    Ok(())
}
