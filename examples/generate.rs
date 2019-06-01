use quicli::prelude::*;
use structopt::StructOpt;

/// Debug OpenAPI stuff
#[derive(Debug, StructOpt)]
struct Cli {
    /// OpenAPI file
    #[structopt(parse(from_os_str))]
    file: std::path::PathBuf,
    /// Where to put it
    #[structopt(parse(from_os_str))]
    outdir: std::path::PathBuf,
    /// What to generate
    #[structopt(
        long = "mode",
        default_value = "rust",
        raw(possible_values = "&Target::variants()")
    )]
    target: Target,
    #[structopt(flatten)]
    verbosity: Verbosity,
}

#[derive(Debug, strum_macros::EnumString, strum_macros::EnumVariantNames)]
#[strum(serialize_all = "kebab_case")]
enum Target {
    Rust,
}

fn main() -> CliResult {
    let args = Cli::from_args();
    args.verbosity.setup_env_logger("openapi_alors")?;
    let content = read_file(&args.file)?;
    let openapi: openapiv3::OpenAPI = serde_yaml::from_str(&content).context("wat")?;

    match args.target {
        Target::Rust => {
            let schemas = openapi_alors::collect_schemas(&openapi)?;
            let tokens = openapi_alors::generate::rust::types(&schemas, &openapi)?;

            let name = args
                .file
                .file_stem()
                .ok_or_else(|| format_err!("file `{}` has no extension", args.file.display()))?;

            let target_file = args.outdir.join(name).with_extension("rs");

            write_to_file(&target_file, &tokens.to_string())?;

            let fmt = std::process::Command::new("rustfmt")
                .arg(&target_file)
                .status()?;

            if !fmt.success() {
                Err(format_err!("rustfmt failed"))?;
            }
        }
    }

    Ok(())
}
