use std::{env, path::PathBuf, process::exit};
use thol_sprites_mini_parser::parser::{parse, types::Object};

fn main() -> anyhow::Result<()> {
    let mut args = env::args();
    args.next();
    let objects_dir = args.next().map(|x| PathBuf::from(&x));

    match objects_dir {
        Some(path) => {
            if path.is_dir() {
                let objects = parse(&path)?;
                let objects_str =
                    serde_json::to_string_pretty::<Vec<Object>>(&objects)?;

                print!("{}", objects_str);
            } else {
                eprintln!(
                    "{} is an invalid objects directory",
                    path.display()
                );
                exit(1);
            }
        }
        None => {
            eprintln!("Need THOL objects directory path as argument");
            exit(1)
        }
    }
    Ok(())
}
