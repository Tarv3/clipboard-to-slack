use std::error::Error;

use arboard::Clipboard;
#[cfg(target_os = "linux")]
use arboard::SetExtLinux;
use clap::{Parser, Subcommand};
#[derive(Parser)]
#[command(author, version, about, long_about=None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    List(List),
}

#[derive(Clone, Parser)]
struct List {
    #[arg(short, long, default_value_t = 2)]
    pub spaces: usize,
}

const DAEMONIZE_ARG: &str = "__internal_daemonize";

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().collect::<Vec<_>>();
    let mut clipboard = Clipboard::new()?;
    let mut text = loop {
        let value = clipboard.get_text();
        if let Err(arboard::Error::ClipboardOccupied) = &value {
            continue;
        }
        break value;
    }?;

    let cli = match args.get(1).map(String::as_str) == Some(DAEMONIZE_ARG) {
        true => {
            args.remove(1);
            Cli::parse_from(args)
        }
        false => {
            if cfg!(target_os = "linux") {
                args.remove(0);
                std::process::Command::new(std::env::current_exe()?)
                    .arg(DAEMONIZE_ARG)
                    .args(args)
                    .stdin(std::process::Stdio::null())
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .current_dir("/")
                    .spawn()?;

                return Ok(());
            }
            Cli::parse_from(args)
        }
    };

    match cli.command {
        Command::List(args) => clipboard_list_to_slack(args.spaces, &mut text),
    }

    #[cfg(target_os="linux")]
    clipboard.set().wait().text(text)?;
    #[cfg(not(target_os="linux"))]
    clipboard.set_text(text)?;

    Ok(())
}

fn clipboard_list_to_slack(spaces_to_tab: usize, text: &mut String) {
    const REPLACE: &str = "    ";
    let mut output = String::new();

    for line in text.lines() {
        let text = match line.find(|c| c != ' ' && c != '\t') {
            Some(idx) => {
                let tabs = &line[..idx];
                let text = &line[idx..];

                let mut count = 0;

                for c in tabs.chars() {
                    match c {
                        ' ' => count += 1,
                        '\t' => {
                            count = 0;
                            output.push_str(REPLACE);
                            continue;
                        }
                        _ => unreachable!(),
                    }

                    if count >= spaces_to_tab {
                        count = 0;
                        output.push_str(REPLACE);
                    }
                }

                text
            }
            None => line,
        };

        output.push_str(text);
        output.push('\n');
    }

    output.pop();
    *text = output;
}
