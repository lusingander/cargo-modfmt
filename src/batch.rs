use std::{
    io::{self, Write},
    path::PathBuf,
    process::ExitCode,
};

use crate::{
    file::{FileError, FileMode, process},
    organize::FormatOptions,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Check,
    Write,
    Files,
}

pub struct Input {
    pub path: PathBuf,
    pub options: FormatOptions,
}

pub fn run(inputs: &[Input], mode: Mode) -> Result<ExitCode, FileError> {
    let mut stdout = io::stdout().lock();
    let mut stderr = io::stderr().lock();
    let mut changed = 0;
    let mut unchanged = 0;
    let mut failed = 0;
    for input in inputs {
        let path = &input.path;
        if mode == Mode::Files {
            writeln!(stdout, "{path:?}")
                .and_then(|()| stdout.flush())
                .map_err(|error| FileError::new(path, "list file", error))?;
            continue;
        }
        let file_mode = if mode == Mode::Write {
            FileMode::Write
        } else {
            FileMode::Check
        };
        match process(path, input.options, file_mode) {
            Ok(true) => {
                changed += 1;
                if mode == Mode::Write {
                    writeln!(stderr, "reformatted: {path:?}")
                        .and_then(|()| stderr.flush())
                        .map_err(|error| FileError::new(path, "report update for", error))?;
                } else {
                    writeln!(stdout, "would reformat: {path:?}")
                        .and_then(|()| stdout.flush())
                        .map_err(|error| FileError::new(path, "report changes for", error))?;
                }
            }
            Ok(false) => unchanged += 1,
            Err(error) => {
                failed += 1;
                writeln!(stderr, "error: {error}")
                    .and_then(|()| stderr.flush())
                    .map_err(|error| FileError::new(path, "report failure for", error))?;
            }
        }
    }
    if failed > 0 {
        if mode == Mode::Write {
            writeln!(stderr, "{changed} files updated; {unchanged} unchanged; {failed} failed; completed updates were not rolled back")
                .and_then(|()| stderr.flush())
                .map_err(|error| FileError::new(&inputs[0].path, "report batch results for", error))?;
        }
        Ok(ExitCode::FAILURE)
    } else if mode == Mode::Check && changed > 0 {
        Ok(ExitCode::from(3))
    } else {
        Ok(ExitCode::SUCCESS)
    }
}
