//! tinct - Theme Injector
//!
//! A theme injector tool that applies Material Design 3 color palettes
//! to various configuration files.

use std::fs;
use std::process;

mod cli;

use clap::Parser;
use tinct::image::SchemeType;
use tinct::{Pipeline, PipelineConfig};

fn main() {
    let args = cli::CliArgs::parse();

    if let Err(e) = args.validate() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }

    // Resolve config path and parse
    let config_path = tinct::resolve_config_file_path(args.config.as_ref());
    let config_content = fs::read_to_string(&config_path).expect("Could not read config file");
    let config_root = tinct::config::ConfigRoot::parse(&config_content)
        .expect("Invalid TOML format in config file");

    // Resolve config directory
    let config_dir = std::path::Path::new(&config_path)
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .to_string_lossy()
        .to_string();

    // Extract algorithm config before moving config_root
    let algorithm = config_root.algorithm.clone();
    let image_config = config_root.image.clone();
    let mut flat_config = config_root.into_flat_config();

    // Resolve paths relative to config file
    for group in flat_config.values_mut() {
        for section in group.values_mut() {
            tinct::path_resolver::resolve_config_paths(section, &config_dir);
        }
    }

    // Determine theme source
    let image_scheme_type = resolve_scheme_type(&args.scheme_type, &image_config.scheme_type);
    let theme_source = if let Some(ref seed) = args.seed {
        tinct::pipeline::ThemeSource::Seed(seed.clone())
    } else if let Some(ref image_path) = args.image {
        tinct::pipeline::ThemeSource::Image {
            path: image_path.clone(),
            scheme_type: image_scheme_type,
        }
    } else {
        tinct::pipeline::ThemeSource::File(args.theme.clone().unwrap())
    };

    // Build pipeline config
    let pipeline_config = PipelineConfig {
        config_path,
        flat_config,
        config_dir,
        mode: args.mode,
        preview: args.preview,
        log_level: args.log_level,
        algorithm,
        image_scheme_type: Some(image_scheme_type),
        theme_source,
    };

    // Run pipeline
    if let Err(e) = Pipeline::run(pipeline_config) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

/// Resolve scheme type: CLI arg > config file > default
fn resolve_scheme_type(
    cli_scheme: &Option<SchemeType>,
    config_scheme: &Option<String>,
) -> SchemeType {
    if let Some(cli) = cli_scheme {
        *cli
    } else if let Some(cfg) = config_scheme {
        SchemeType::parse(cfg).unwrap_or(SchemeType::TonalSpot)
    } else {
        SchemeType::TonalSpot
    }
}
