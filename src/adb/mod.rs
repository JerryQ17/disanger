pub mod command;
pub mod envs;
pub mod error;
pub mod global_options;
pub mod socket;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::{fs, io};

#[derive(Debug)]
pub struct Adb {
    /// The canonical directory where the adb binary is located.
    /// If None, the adb binary should be in PATH.
    dir: Option<PathBuf>,
}

impl Adb {
    ///
    pub fn new<P: AsRef<Path>>(dir: Option<P>) -> io::Result<Self> {
        Ok(Self {
            dir: Self::check_dir(dir.as_ref().map(|d| d.as_ref()))?,
        })
    }

    /// The canonical directory where the adb binary is located.
    ///
    /// If [`None`], uses the adb binary in PATH.
    pub fn dir(&self) -> Option<&Path> {
        self.dir.as_ref().map(|p| p.as_ref())
    }

    /// Sets the directory where the adb binary is located.
    ///
    /// If `dir` is Some, check if adb binary is in `dir`.
    /// Otherwise, check if adb binary is in PATH.
    /// In either case, if adb binary is not found, return an error.
    ///
    /// # Errors
    ///
    /// Including but not limited to:
    ///
    /// - [`io::ErrorKind::InvalidInput`]: `dir` is not a valid directory.
    /// - [`io::ErrorKind::NotFound`]: adb binary not found in `dir` or PATH.
    pub fn set_dir<P: AsRef<Path>>(&mut self, dir: Option<P>) -> io::Result<&mut Self> {
        let dir = dir.as_ref().map(|d| d.as_ref());
        if dir != self.dir.as_deref() {
            self.dir = Self::check_dir(dir)?;
        }
        Ok(self)
    }

    /// Checks if the adb binary is in `dir`.
    ///
    /// # Errors
    ///
    /// If `dir` is [`None`]:
    ///
    /// - [`io::ErrorKind::NotFound`]: adb binary not found in PATH.
    ///
    /// If `dir` is [`Some`]:
    ///
    /// - [`io::ErrorKind::InvalidInput`]: `dir` is not a valid directory.
    fn check_dir(dir: Option<&Path>) -> io::Result<Option<PathBuf>> {
        let dir = match dir {
            None => None,
            Some(dir) => Some(fs::canonicalize(dir)?),
        };
        match dir {
            None => {
                // `dir` is None, trying to search in PATH.
                if Command::new("adb").output().is_err() {
                    return Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        "adb binary not found in PATH. \
                        Consider set `dir` explicitly or add it to PATH.",
                    ));
                }
            }
            Some(ref dir) => {
                // `dir` is `Some`, checking whether it's a valid directory.
                // If the path doesn't exist, `fs::metadata()` will return an error .
                // So we just need to check `is_dir()`.
                if !fs::metadata(dir)?.is_dir() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("`dir` is not a valid directory: {}", dir.display()),
                    ));
                }
                // Search for adb binary in `dir`.
                if Command::new("adb").current_dir(dir).output().is_err() {
                    return Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        format!("adb binary not found in `dir`: {}", dir.display()),
                    ));
                }
            }
        }
        Ok(dir)
    }
    //
    // /// Returns a [`Command`] that runs adb shell on the client.
    // pub fn command(&self) -> Command {
    //     let mut cmd = Command::new("adb");
    //     cmd.arg("-s").arg(&self.name);
    //     if let Some(dir) = &self.dir {
    //         cmd.current_dir(dir);
    //     }
    //     cmd
    // }
    //
    // pub fn shell(&self) -> Command {
    //     let mut cmd = self.command();
    //     cmd.arg("shell");
    //     cmd
    // }
}
