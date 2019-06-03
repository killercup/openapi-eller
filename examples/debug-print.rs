use quicli::prelude::*;
use structopt::StructOpt;

/// Debug OpenAPI stuff
#[derive(Debug, StructOpt)]
struct Cli {
    /// OpenAPI file
    #[structopt(parse(from_os_str))]
    file: std::path::PathBuf,
    /// What to print
    #[structopt(
        long = "mode",
        default_value = "Mode::Schemas",
        raw(possible_values = "&Mode::variants()")
    )]
    mode: Mode,
    #[structopt(flatten)]
    verbosity: Verbosity,
}

#[derive(Debug, strum_macros::EnumString, strum_macros::EnumVariantNames)]
#[strum(serialize_all = "kebab_case")]
enum Mode {
    All,
    Schemas,
    SchemasFull,
}

fn main() -> CliResult {
    let args = Cli::from_args();
    args.verbosity.setup_env_logger("openapi_alors")?;
    let content = read_file(&args.file)?;
    let openapi: openapiv3::OpenAPI = serde_yaml::from_str(&content).context("wat")?;

    match args.mode {
        Mode::All => println!("{:#?}", openapi),
        Mode::Schemas => println!(
            "{}",
            openapi_alors::collect_schemas(&openapi)?.keys().collect::<Vec<_>>().join("\n")
        ),
        Mode::SchemasFull => println!("{:#?}", openapi_alors::collect_schemas(&openapi)?),
    }

    Ok(())
}
