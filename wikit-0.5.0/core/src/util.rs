use crate::elog;
use crate::error::{AnyResult, Context};

use std::process::Command;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::io::{Error, ErrorKind};
use std::net::TcpListener;

struct ArgParser<'a> {
    buf: &'a str,
    consumed: usize,
    total: usize,
}

impl<'a> ArgParser<'a> {
    fn new(buf: &'a str) -> Self {
        let buf = buf.trim();
        ArgParser {
            buf: buf,
            consumed: 0,
            total: buf.chars().count(),
        }
    }
}

impl<'a> Iterator for ArgParser<'a> {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        if self.consumed >= self.total {
            return None;
        }

        let mut arg = String::new();
        let (mut quote, mut quotech) = (false, ' ');
        // Mark the ends of an argument parsing
        let mut spaced = false;
        for c in self.buf.chars().skip(self.consumed) {
            self.consumed += 1;
            if quote || c == '\'' || c == '\"' {
                if spaced {
                    self.consumed -= 1;
                    break;
                }
                // Take until the next same `c` or end
                if quote {
                    if c == quotech {
                        quote = false;
                    } else {
                        // Note that spaces in quote will also go here and keeped as they were
                        arg.push(c);
                    }
                } else {
                    quote = true;
                    quotech = c;
                }
            } else if c == ' ' {
                // Discard any non-quote repeted space
                if !spaced {
                    spaced = true;
                }
            } else {
                if spaced {
                    // All repeted spaces are consumed, and we have consumed once more non-spaced
                    // char, so we should end the parsing and go back one position. Notice that
                    // only non-quote space will go here, since all quote space will go into the
                    // first if branch.
                    self.consumed -= 1;
                    break;
                } else {
                    arg.push(c);
                }
            }
        }
        return Some(arg);
    }
}

pub fn runcmd(cmd: &str, envs: Option<Vec<(String, String)>>) -> AnyResult<String> {
    let argparser = ArgParser::new(cmd);
    let cmd: Vec<String> = argparser.into_iter().collect();
    let envs: HashMap<String, String> = if let Some(envs) = envs {
        envs.into_iter().collect()
    } else {
        HashMap::new()
    };
    let outbuf = match cmd.len() {
        0 => return Err(elog!("Empty command")),
        1 => Command::new(&cmd[0]).envs(&envs).output(),
        _ => Command::new(&cmd[0]).envs(&envs).args(&cmd[1..]).output(),
    };
    let outbuf = outbuf.context(elog!("failed to run command {:?}", cmd))?;
    if !outbuf.status.success() {
        let err = match std::str::from_utf8(&outbuf.stderr[..]) {
            Ok(e) => e.to_string(),
            Err(_) => format!("command exit with error: {:?}", outbuf.stderr),
        };
        return Err(elog!("{}", err));
    }
    let output = std::str::from_utf8(&outbuf.stdout[..])
        .context(elog!("failed to decode output: {:?}", outbuf.stdout))?;
    Ok(output.to_string())
}

/// parse_path returns a three-element tuple which is `(parentdir, stem, suffix)`
pub fn parse_path<P>(filepath: P) -> AnyResult<(PathBuf, String, String)> where P: AsRef<Path> {
    let filepath = filepath.as_ref();
    let stdpath = std::fs::canonicalize(filepath)
        .context(elog!("filepath [{}] is not exist", filepath.display()))?;
    let parentdir = stdpath.parent()
        .context(elog!("cannot get parent directory of {}", stdpath.display()))?;
    let stem = stdpath.file_stem()
        .context(elog!("cannot get file stem from [{}]", stdpath.display()))?
        .to_str().context(elog!("cannot convert osstr to str"))?;
    let suffix = stdpath.extension()
        .context(elog!("cannot get extension of [{}]", stdpath.display()))?
        .to_str().context(elog!("cannot convert osstr to str"))?;

    Ok((parentdir.to_path_buf(), stem.to_string(), suffix.to_string()))
}

pub fn filter_file_by_suffix<P>(path: P, suffix: &str) -> Option<Vec<PathBuf>> where P: AsRef<Path> {
    let path = path.as_ref();
    let list = path.read_dir().and_then(|files| {
        let list = files
            .filter_map(|d| -> Option<PathBuf> {
                if let Ok(entry) = d {
                    let entpath = entry.path();
                    if let Some(ext) = entpath.extension() {
                        if ext == suffix {
                            return Some(entpath);
                        }
                    }
                }
                return None;
            })
            .collect::<Vec<PathBuf>>();
        if list.len() > 0 {
            Ok(list)
        } else {
            Err(Error::new(ErrorKind::NotFound, ""))
        }
    });
    list.ok()
}

pub fn normalize_word<S>(word: S) -> String where S: AsRef<str> {
    let word = word.as_ref().trim_matches(|c: char| c.is_control() || c.is_whitespace());
    word.to_lowercase()
}

// Get available TCP port
//
// If `default_port` is some, then check if `default_port` is available, if yes then return this
// port, otherwise pick one port from [6000, 9000).
pub fn get_free_tcp_port(default_port: Option<u16>) -> Option<u16> {
    if let Some(port) = default_port {
        if TcpListener::bind(("127.0.0.1", port)).is_ok() {
            return default_port
        }
    }
    (6000..9000).find(|port| TcpListener::bind(("127.0.0.1", *port)).is_ok())
}

#[test]
fn test_argparser() {
    let cmd = "a bc def";
    let mut argparser = ArgParser::new(cmd);
    assert_eq!(Some("a".into()), argparser.next());
    assert_eq!(Some("bc".into()), argparser.next());
    assert_eq!(Some("def".into()), argparser.next());

    let cmd = " a bc  def   ghi ";
    let mut argparser = ArgParser::new(cmd);
    assert_eq!(Some("a".into()), argparser.next());
    assert_eq!(Some("bc".into()), argparser.next());
    assert_eq!(Some("def".into()), argparser.next());
    assert_eq!(Some("ghi".into()), argparser.next());

    let cmd = " a bc 'def ghi' 'jkl  mno' 'pqr   st' ' x y z  ' ";
    let mut argparser = ArgParser::new(cmd);
    assert_eq!(Some("a".into()), argparser.next());
    assert_eq!(Some("bc".into()), argparser.next());
    assert_eq!(Some("def ghi".into()), argparser.next());
    assert_eq!(Some("jkl  mno".into()), argparser.next());
    assert_eq!(Some("pqr   st".into()), argparser.next());
    assert_eq!(Some(" x y z  ".into()), argparser.next());
}

#[test]
fn parse_path_test() {
    let p = parse_path("test/demo.txt");
    if let Ok(p) = p {
        assert_eq!(p.1.as_str(), "demo");
        assert_eq!(p.2.as_str(), "txt");
    } else {
        assert_eq!(true, false);
    }
}

