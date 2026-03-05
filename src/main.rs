use clap::Parser;
use templar_cli::commands::Cli;
use templar_cli::display::banner;
use templar_cli::display::BannerOpts;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("templar_cli=info".parse().expect("valid log directive")),
        )
        .init();

    let cli = Cli::parse();

    if cli.should_show_banner() {
        let banner_opts = BannerOpts {
            quiet: cli.global_opts.quiet,
            color: cli.global_opts.color.clone(),
            output: cli.global_opts.output.clone(),
            no_banner: cli.global_opts.no_banner,
            no_animation: cli.global_opts.no_animation,
        };
        banner::print_banner(&banner_opts);
    }

    if let Err(e) = cli.run().await {
        eprintln!("{e}");
        std::process::exit(e.exit_code());
    }
}
