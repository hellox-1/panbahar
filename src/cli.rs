use inquire::Select;
use std::env;
use std::path::PathBuf;

pub fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        print_usage();
        return Ok(());
    }

    let target = match args.as_slice() {
        [] => select_target()?,
        [target] => PathBuf::from(target),
        _ => {
            print_usage();
            return Err("too many arguments".to_string());
        }
    };

    crate::files::validate_target(&target)?;
    crate::editor::run(&target)
}

fn select_target() -> Result<PathBuf, String> {
    let root = env::current_dir().map_err(|error| error.to_string())?;
    let files = crate::files::find_react_files(&root).map_err(|error| error.to_string())?;
    if files.is_empty() {
        return Err("no .tsx or .jsx files found in the current directory".to_string());
    }

    let choices: Vec<String> = files
        .iter()
        .map(|file| {
            file.strip_prefix(&root)
                .unwrap_or(file)
                .display()
                .to_string()
        })
        .collect();
    let choice = Select::new("Choose a React file:", choices)
        .prompt()
        .map_err(|error| error.to_string())?;

    Ok(root.join(choice))
}

fn print_usage() {
    println!("panbahar - write a blog live into a React component");
    println!();
    println!("Usage:");
    println!("  panbahar [TARGET]");
    println!("  panbahar                    # choose a .tsx/.jsx file interactively");
    println!("  panbahar blog.tsx           # open the live editor for blog.tsx");
    println!();
    println!("Keys:");
    println!("  Esc    exit");
    println!("  Ctrl-S save explicitly");
    println!("  Arrows move the cursor");
}
