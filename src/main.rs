mod cli;
mod editor;
mod files;
mod renderer;

fn main() {
    if let Err(error) = cli::run() {
        eprintln!("panbahar: {error}");
        std::process::exit(1);
    }
}
