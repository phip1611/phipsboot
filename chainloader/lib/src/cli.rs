//! CLI parsing of the loader. The CLI looks like this:
//!
//! `[--load=modul-id-if-kernel] [--loggers=serial,debugcon]`

use ::regex::Regex;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::str::FromStr;

mod regex {
    pub const LOAD: &str = "--load=(?P<load>[A-z0-9-_.]+)+";
    pub const LOGGERS: &str = "--loggers=(?P<loggers>[a-z]+(,[a-z]+)*)?";
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum SupportedLogger {
    Debugcon,
    Serial,
}

impl FromStr for SupportedLogger {
    type Err = ();

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        match name {
            "debugcon" => Ok(Self::Debugcon),
            "serial" => Ok(Self::Serial),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Default)]
pub struct CliArgs {
    loggers: Vec<SupportedLogger>,
    load: String,
}

impl FromStr for CliArgs {
    type Err = ();

    fn from_str(cmdline: &str) -> Result<Self, Self::Err> {
        let mut args = CliArgs::default();

        let regex_load = Regex::new(regex::LOAD).unwrap();
        let regex_loggers = Regex::new(regex::LOGGERS).unwrap();

        if let Some(mtch) = regex_load.captures(cmdline) {
            args.load = mtch
                .name("load")
                .map(|m| m.as_str())
                .unwrap_or("")
                .to_string();
        }

        if let Some(mtch) = regex_loggers.captures(cmdline) {
            let mtch = mtch.name("loggers").map(|m| m.as_str()).unwrap_or("");
            for logger in mtch
                .split(',')
                .filter_map(|mtch| SupportedLogger::from_str(mtch).ok())
            {
                args.loggers.push(logger);
            }
        }

        Ok(args)
    }
}

#[cfg(test)]
mod tests {
    use crate::cli::{CliArgs, SupportedLogger};
    use core::str::FromStr;
    use std::dbg;

    #[test]
    fn test_cli_empty() {
        let cmdline = "";
        let args = CliArgs::from_str(cmdline).unwrap();
        assert_eq!(args.load.as_str(), "");
        assert!(args.loggers.is_empty());
    }

    #[test]
    fn test_cli_normal() {
        let cmdline = "--load=foobar --loggers=serial";
        let args = CliArgs::from_str(cmdline).unwrap();
        assert_eq!(args.load.as_str(), "foobar");
        assert_eq!(args.loggers, [SupportedLogger::Serial]);

        let cmdline = "--load=foobar --loggers=serial,debugcon";
        let args = CliArgs::from_str(cmdline).unwrap();
        assert_eq!(args.load.as_str(), "foobar");
        assert_eq!(
            args.loggers,
            [SupportedLogger::Serial, SupportedLogger::Debugcon]
        );
    }
}
