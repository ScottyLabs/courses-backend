use git_conventional::Commit;
use std::{env, fs, process};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: git-conventional-check <commit-msg-file>");
        process::exit(1);
    }

    let commit_msg_file = &args[1];
    let commit_msg = match fs::read_to_string(commit_msg_file) {
        Ok(msg) => msg,
        Err(e) => {
            eprintln!("Failed to read commit message file: {e}");
            process::exit(1);
        }
    };

    match Commit::parse(&commit_msg) {
        Ok(_) => process::exit(0),
        Err(e) => {
            eprintln!("Invalid commit format: {e}");
            eprintln!("Expected format: <type>[optional scope]: <description>");
            eprintln!("Examples:");
            eprintln!("  feat: add user authentication");
            eprintln!("  fix(api): handle edge case in login flow");
            process::exit(1);
        }
    }
}
