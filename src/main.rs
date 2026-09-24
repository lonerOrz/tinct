//! tinct - Theme Injector
//!
//! A theme injector tool that applies Material Design 3 color palettes
//! to various configuration files.

use std::process;

mod cli;

use clap::Parser;
use tinct::image::SchemeType;
use tinct::pipeline::ThemeSource;
use tinct::{Config, ConfigLoad, Pipeline, PipelineConfig};

fn main() {
    let args = cli::CliArgs::parse();

    // One call resolves the config path, reads, parses and canonicalizes paths.
    // On a fresh system it also scaffolds a commented default config.
    let config = match Config::load_or_init(args.config.as_deref()) {
        Ok(ConfigLoad::Loaded(config)) => *config,
        Ok(ConfigLoad::CreatedDefault(path)) => {
            println!("Created a default config at {}", path.display());
            println!("Add a [templates.<name>] section, then run tinct again.");
            return;
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };

    let scheme_type = resolve_scheme_type(args.scheme_type, config.image.scheme_type);

    // ArgGroup guarantees exactly one source is present.
    let theme_source = match (args.seed, args.image, args.theme) {
        (Some(seed), _, _) => ThemeSource::Seed(seed),
        (_, Some(image), _) => ThemeSource::Image(image),
        (_, _, Some(theme)) => ThemeSource::File(theme),
        (None, None, None) => {
            eprintln!("Error: Exactly one of --theme, --seed, or --image is required.");
            process::exit(1);
        }
    };

    let pipeline_config = PipelineConfig {
        config,
        mode: args.mode,
        preview: args.preview,
        log_level: args.log_level,
        scheme_type,
        theme_source,
    };

    if let Err(e) = Pipeline::run(pipeline_config) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

/// Resolve scheme type: CLI arg > config file > default.
fn resolve_scheme_type(
    cli_scheme: Option<SchemeType>,
    config_scheme: Option<SchemeType>,
) -> SchemeType {
    cli_scheme
        .or(config_scheme)
        .unwrap_or(SchemeType::TonalSpot)
}
