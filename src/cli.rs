//! Small shared command-line helpers for the `ll image` and `ll video`
//! subcommands: flag/positional parsing, value coercion and name normalization.
//!
//! Flags may be given in any order (`--flag value`, `--flag=value`), negative
//! numbers are treated as values rather than flags, and every extra positional
//! argument after `<in> <out>` can stand in for the command's main argument.

use std::collections::{HashMap, HashSet};
use std::io;

#[derive(Default, Debug)]
pub struct Opts {
    pub pos: Vec<String>,
    pub vals: HashMap<String, String>,
    pub bools: HashSet<String>,
}

impl Opts {
    pub fn get(&self, names: &[&str]) -> Option<&str> {
        names.iter().find_map(|n| self.vals.get(*n).map(String::as_str))
    }

    pub fn has(&self, names: &[&str]) -> bool {
        names.iter().any(|n| self.bools.contains(*n))
    }
}

/// Merge two flag lists, dropping duplicates.
pub fn merge(a: &[&'static str], b: &[&'static str]) -> Vec<&'static str> {
    let mut v = a.to_vec();
    for x in b {
        if !v.contains(x) {
            v.push(x);
        }
    }
    v
}

pub fn parse_opts(args: &[String], value_flags: &[&str], bool_flags: &[&str]) -> io::Result<Opts> {
    let mut o = Opts::default();
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        // Negative numbers (e.g. `brightness in out -30`) are positional values,
        // not flags — flag values themselves are consumed before this check.
        if a.starts_with('-') && a.len() > 1 && a.parse::<f64>().is_err() {
            let (name, inline) = match a.find('=') {
                Some(p) => (&a[..p], Some(a[p + 1..].to_string())),
                None => (a.as_str(), None),
            };
            if value_flags.contains(&name) {
                let v = match inline {
                    Some(v) => v,
                    None => {
                        i += 1;
                        args.get(i)
                            .ok_or_else(|| {
                                io::Error::new(
                                    io::ErrorKind::InvalidInput,
                                    format!("missing value for {name}"),
                                )
                            })?
                            .clone()
                    }
                };
                o.vals.insert(name.to_string(), v);
            } else if bool_flags.contains(&name) {
                o.bools.insert(name.to_string());
            } else {
                let mut accepted: Vec<&str> = value_flags.to_vec();
                accepted.extend_from_slice(bool_flags);
                accepted.sort_unstable();
                return err(format!(
                    "unknown flag '{a}'\n  accepted: {}",
                    accepted.join(", ")
                ));
            }
        } else {
            o.pos.push(a.clone());
        }
        i += 1;
    }
    Ok(o)
}

pub fn err<T>(msg: impl Into<String>) -> io::Result<T> {
    Err(io::Error::new(io::ErrorKind::InvalidInput, msg.into()))
}

pub fn req<'a>(o: &'a Opts, names: &[&str], msg: &str) -> io::Result<&'a str> {
    o.get(names)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, msg.to_string()))
}

pub fn pos<'a>(o: &'a Opts, i: usize, what: &str) -> io::Result<&'a str> {
    o.pos
        .get(i)
        .map(String::as_str)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, format!("missing {what}")))
}

/// A flag value, falling back to a positional argument (extra args after in/out).
pub fn arg_or<'a>(o: &'a Opts, names: &[&str], idx: usize) -> Option<&'a str> {
    o.get(names).or_else(|| o.pos.get(idx).map(String::as_str))
}

/// Lowercase a name with all separators stripped, so `Grey-Scale` == `greyscale`.
pub fn normalize(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

pub fn parse_f64(s: &str, what: &str) -> io::Result<f64> {
    s.trim()
        .parse::<f64>()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, format!("{what} is not a number: '{s}'")))
}

pub fn parse_u32(s: &str, what: &str) -> io::Result<u32> {
    s.trim()
        .parse::<u32>()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, format!("{what} is not a number: '{s}'")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn parses_flags_and_positionals() {
        let o = parse_opts(
            &args(&["in.mp4", "out.mp4", "-s", "10", "--crf=18", "--reencode", "-v", "-2"]),
            &["--crf", "-s", "--start", "--value", "-v"],
            &["--reencode"],
        )
        .unwrap();
        assert_eq!(o.pos, vec!["in.mp4", "out.mp4"]);
        assert_eq!(o.get(&["-s"]), Some("10"));
        assert_eq!(o.get(&["--crf"]), Some("18"));
        assert!(o.has(&["--reencode"]));
        // a negative number is a value, not a flag
        assert_eq!(o.get(&["--value", "-v"]), Some("-2"));
    }

    #[test]
    fn rejects_unknown_flags_with_a_hint() {
        let e = parse_opts(&args(&["--widht", "5"]), &["--width"], &[]).unwrap_err();
        assert!(e.to_string().contains("unknown flag '--widht'"));
        assert!(e.to_string().contains("--width"));
    }

    #[test]
    fn falls_back_to_positionals() {
        let o = parse_opts(&args(&["a.png", "b.png", "400x300+1+2"]), &[], &[]).unwrap();
        assert_eq!(arg_or(&o, &["--geometry", "-g"], 2), Some("400x300+1+2"));
        assert_eq!(arg_or(&o, &["--nope"], 5), None);
    }

    #[test]
    fn normalizes_names() {
        assert_eq!(normalize("Grey-Scale"), "greyscale");
        assert_eq!(normalize("MP4"), "mp4");
        assert_eq!(merge(&["-a"], &["-a", "-b"]), vec!["-a", "-b"]);
    }
}
