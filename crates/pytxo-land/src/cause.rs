//! Why branches that pass their tests alone fail them together: a name one
//! branch renamed or removed that another branch's new lines still use.
//!
//! Definitions come from `pytxo-signal`, uses from each branch's own diff,
//! and `git grep` tells a name a branch stopped using everywhere from one it
//! only moved. The cause is a reading of the runs, not a record of them, so
//! it stays out of the receipt and its digest. A git call that fails or a cap
//! that is reached means no cause, never a guess.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::Output;
use std::time::Instant;

use pytxo_signal::{name_use, parses, symbol_spans, uses_name};
use serde::Serialize;

use crate::git::Git;
use crate::render::rank;

/// Changed files of one input read for the names they define.
const MAX_FILES: usize = 200;
/// Larger files are not parsed.
const MAX_FILE_BYTES: usize = 1 << 20;
/// Names one input stopped defining that another input's new lines use.
const MAX_GONE: usize = 50;
/// `git grep` calls in all, the dear part over a big tree.
const MAX_GREPS: usize = 100;
/// Shorter names are too common to blame.
const MIN_NAME: usize = 3;

/// One branch renamed or removed `name`, and another branch's new lines
/// still use it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cause {
    /// The input that renamed or removed `name`, by its position in the inputs.
    pub by: usize,
    /// The input whose new lines still use `name`.
    pub user: usize,
    pub name: String,
    /// What `by` calls it now, when its diff shows the rename.
    pub renamed_to: Option<String>,
    /// Where `user`'s new lines use it, numbered as on that branch.
    pub uses: Vec<Use>,
    /// The failing output names it, or the place of one of the uses.
    pub confirmed: bool,
    /// The other inputs whose new lines use it too, in input order: as much
    /// to blame as `user`, whom the runs on the pair do not clear.
    pub also: Vec<usize>,
}

/// A line of a file as one branch has it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Use {
    pub path: String,
    pub line: u32,
}

/// A name one input stopped defining, by its position among the merged
/// inputs: what it calls the name now, when its diff says, and whether one
/// it defined was a member, such as a method, which code calls as `a.name`.
struct Gone {
    by: usize,
    name: String,
    renamed: Option<String>,
    member: bool,
}

/// A merged input: its position in the inputs, the commit it shares with the
/// target, and its own commit.
pub(crate) struct Merged<'a> {
    pub index: usize,
    pub merge_base: &'a str,
    pub commit: &'a str,
}

/// One file's part of a diff, lines numbered as on each side.
#[derive(Default)]
struct FileDiff {
    path: String,
    /// The file exists before the change.
    before: bool,
    /// The file exists after it.
    after: bool,
    removed: Vec<(u32, String)>,
    added: Vec<(u32, String)>,
    /// The whole file before and after, for one to parse that the diff shows
    /// whole and that is at most `MAX_FILE_BYTES`.
    old_source: Option<String>,
    new_source: Option<String>,
}

/// The cause of a combined failure that every merged input passed alone,
/// from the merged `inputs`, the `combined` commit they were checked at, and
/// the output of its failing runs. `None` as well once `deadline` passes:
/// no git call starts after it, and one still going stops.
pub(crate) fn find(
    git: &Git,
    inputs: &[Merged],
    combined: &str,
    failing: &[&[String]],
    deadline: Option<Instant>,
) -> Option<Cause> {
    // The git calls of each step run side by side, eight at a time, so the
    // search takes about four calls' time: the diffs, the greps of the
    // inputs, the greps of the combined change and the reads of what they
    // find.
    let calls: Vec<Vec<String>> = inputs
        .iter()
        .map(|i| diff_args(i.merge_base, i.commit))
        .collect();
    let diffs: Vec<Vec<FileDiff>> = git
        .outputs_stoppable(&calls, deadline)
        .ok()?
        .iter()
        .map(|out| {
            out.status
                .success()
                .then(|| parse_diff(&String::from_utf8_lossy(&out.stdout)))
        })
        .collect::<Option<_>>()?;
    // The words of each input's new lines: a name none of the others' new
    // lines has cannot break them, so it costs no grep.
    let words: Vec<BTreeSet<&str>> = diffs
        .iter()
        .map(|d| {
            d.iter()
                .flat_map(|f| f.added.iter().flat_map(|(_, l)| words_of(l)))
                .collect()
        })
        .collect();
    // Names an input stopped defining that another input's new lines use,
    // with the input and what it calls the name now, when its diff says.
    let mut candidates: Vec<Gone> = Vec::new();
    for (b, diff) in diffs.iter().enumerate() {
        let used_by_another =
            |name: &str| (0..inputs.len()).any(|u| u != b && words[u].contains(name));
        let files: Vec<&FileDiff> = diff
            .iter()
            .filter(|f| f.before && !f.removed.is_empty() && parses(Path::new(&f.path)))
            .collect();
        if files.len() > MAX_FILES {
            return None;
        }
        let mut looked_at = BTreeSet::new();
        let mut gone = 0;
        for file in files {
            let Some(before) = &file.old_source else {
                continue;
            };
            let names: Vec<(String, bool)> = defined(&file.path, before)
                .into_iter()
                .filter(|(n, _)| used_by_another(n) && looked_at.insert(n.clone()))
                .collect();
            if names.is_empty() {
                continue;
            }
            let now: Vec<String> = match &file.new_source {
                Some(after) => defined(&file.path, after)
                    .into_iter()
                    .map(|(n, _)| n)
                    .collect(),
                // Too big to tell what it still defines: nothing from it.
                None if file.after => continue,
                None => Vec::new(),
            };
            for (name, member) in names.into_iter().filter(|(n, _)| !now.contains(n)) {
                gone += 1;
                if gone > MAX_GONE {
                    return None;
                }
                let renamed = renamed_to(diff, file, &name, &now);
                candidates.push(Gone {
                    by: b,
                    name,
                    renamed,
                    member,
                });
            }
        }
    }
    // Gone only when the input no longer uses the name itself: not a word
    // like `get`. The files it changed, which the diff shows whole, are read
    // for code that says the name, where a string or a comment is no use,
    // such as the `require('./price')` a rename of the function `price`
    // leaves behind; the grep looks for the word everywhere else.
    let mut grep = Grep {
        git,
        calls: 0,
        deadline,
    };
    let left_out: Vec<Option<Vec<String>>> = candidates
        .iter()
        .map(|c| said_only(&diffs[c.by], &c.name))
        .collect();
    let asks: Vec<(&str, &str, &[String])> = candidates
        .iter()
        .zip(&left_out)
        .filter_map(|(c, files)| Some((inputs[c.by].commit, c.name.as_str(), files.as_deref()?)))
        .collect();
    let mut mentioned = grep.mentions(&asks)?.into_iter();
    let gone: Vec<&Gone> = candidates
        .iter()
        .zip(&left_out)
        .filter(|(_, files)| files.is_some() && mentioned.next() == Some(false))
        .map(|(candidate, _)| candidate)
        .collect();
    let names: BTreeSet<&str> = gone.iter().map(|c| c.name.as_str()).collect();
    let names: Vec<&str> = names.into_iter().collect();
    let still_defined: BTreeMap<&str, bool> = names
        .iter()
        .copied()
        .zip(defines(git, &mut grep, combined, &names)?)
        .collect();
    let mut found = Vec::new();
    for c in gone {
        if still_defined[c.name.as_str()] {
            continue;
        }
        for (u, user) in inputs.iter().enumerate().filter(|&(u, _)| u != c.by) {
            let uses = uses_in(&diffs[u], &c.name, c.member);
            if uses.is_empty() {
                continue;
            }
            found.push(Cause {
                by: inputs[c.by].index,
                user: user.index,
                confirmed: confirms(failing, &c.name, &uses),
                name: c.name.clone(),
                renamed_to: c.renamed.clone(),
                uses,
                also: Vec::new(),
            });
        }
    }
    // Confirmed first, then the most uses; a stable sort keeps input order.
    found.sort_by_key(|c| (!c.confirmed, std::cmp::Reverse(c.uses.len())));
    let mut found = found.into_iter();
    let mut cause = found.next()?;
    cause.also = found
        .filter(|c| c.by == cause.by && c.name == cause.name)
        .map(|c| c.user)
        .collect();
    cause.also.sort_unstable();
    Some(cause)
}

/// `git grep` within the cap on calls and the deadline. `None` past either
/// or on an error.
struct Grep<'a> {
    git: &'a Git,
    calls: usize,
    deadline: Option<Instant>,
}

impl Grep<'_> {
    /// Whether each commit mentions its name as a word, binary files aside,
    /// in a file the pathspecs given with it do not leave out.
    fn mentions(&mut self, asks: &[(&str, &str, &[String])]) -> Option<Vec<bool>> {
        let calls = asks.iter().map(|&(commit, name, left_out)| {
            let mut args = vec!["-q", "-e", name, commit, "--"];
            if !left_out.is_empty() {
                args.push(".");
                args.extend(left_out.iter().map(String::as_str));
            }
            args
        });
        self.run(calls)?
            .iter()
            .map(|out| match out.status.code() {
                Some(0) => Some(true),
                Some(1) => Some(false),
                _ => None,
            })
            .collect()
    }

    /// The files of `commit` that mention each of `names` as a word.
    fn files(&mut self, commit: &str, names: &[&str]) -> Option<Vec<Vec<String>>> {
        let calls = names
            .iter()
            .map(|&name| vec!["-l", "-z", "-e", name, commit, "--"]);
        let prefix = format!("{commit}:");
        self.run(calls)?
            .iter()
            .map(|out| match out.status.code() {
                Some(0) => Some(
                    String::from_utf8_lossy(&out.stdout)
                        .split('\0')
                        .filter_map(|p| p.strip_prefix(prefix.as_str()))
                        .map(str::to_string)
                        .collect(),
                ),
                Some(1) => Some(Vec::new()),
                _ => None,
            })
            .collect()
    }

    fn run<'s>(&mut self, calls: impl Iterator<Item = Vec<&'s str>>) -> Option<Vec<Output>> {
        // Whole words, the name as it is, whatever grep.* the user set.
        let calls: Vec<Vec<String>> = calls
            .map(|args| {
                ["grep", "-w", "-F", "-I", "--no-color"]
                    .into_iter()
                    .chain(args)
                    .map(str::to_string)
                    .collect()
            })
            .collect();
        self.calls += calls.len();
        if self.calls > MAX_GREPS {
            return None;
        }
        self.git.outputs_stoppable(&calls, self.deadline).ok()
    }
}

/// Whether `commit` defines each of `names` in one of the files that mention
/// it: a symbol of that name, or another binding outside every function and
/// block that is no import, such as `const price = memo(...)`,
/// `exports.price = ...` or a Python `price = ...`, which the code that
/// imports the file reaches as well. A file too big to parse counts as one
/// that does, since it may.
fn defines(git: &Git, grep: &mut Grep, commit: &str, names: &[&str]) -> Option<Vec<bool>> {
    let files = grep.files(commit, names)?;
    let paths: BTreeSet<&str> = files
        .iter()
        .flatten()
        .map(String::as_str)
        .filter(|p| parses(Path::new(p)))
        .collect();
    if paths.len() > MAX_FILES {
        return None;
    }
    let calls: Vec<Vec<String>> = paths
        .iter()
        .map(|p| vec!["cat-file".into(), "blob".into(), format!("{commit}:{p}")])
        .collect();
    // Each file and the symbols it defines, `None` for one too big to parse.
    let mut read: BTreeMap<&str, Option<(String, Vec<String>)>> = BTreeMap::new();
    let outs = git.outputs_stoppable(&calls, grep.deadline).ok()?;
    for (path, out) in paths.into_iter().zip(outs) {
        if !out.status.success() {
            return None;
        }
        let source = (out.stdout.len() <= MAX_FILE_BYTES)
            .then(|| String::from_utf8_lossy(&out.stdout).into_owned());
        let symbols = |s: &str| defined(path, s).into_iter().map(|(n, _)| n).collect();
        read.insert(
            path,
            source.map(|s| {
                let symbols = symbols(&s);
                (s, symbols)
            }),
        );
    }
    let binds = |path: &str, name: &str| match read.get(path) {
        None => false,
        Some(None) => true,
        Some(Some((source, symbols))) => {
            symbols.iter().any(|n| n == name)
                || name_use(Path::new(path), source, name, false).is_some_and(|u| u.exports)
        }
    };
    Some(
        names
            .iter()
            .zip(&files)
            .map(|(name, files)| files.iter().any(|p| binds(p, name)))
            .collect(),
    )
}

/// The names `source` defines, in order: the last part of each symbol's
/// qualified name (`Cart.total` is `total`), when it is a plain word of
/// three characters or more, and whether one of that name is a member, such
/// as a method.
fn defined(path: &str, source: &str) -> Vec<(String, bool)> {
    let mut names: Vec<(String, bool)> = Vec::new();
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    for span in symbol_spans(Path::new(path), source) {
        let Some(last) = span.name.rsplit(['.', ':']).next().filter(|l| is_name(l)) else {
            continue;
        };
        match seen.get(last) {
            Some(&at) => names[at].1 |= span.member,
            None => {
                seen.insert(last.to_string(), names.len());
                names.push((last.to_string(), span.member));
            }
        }
    }
    names
}

fn is_name(word: &str) -> bool {
    word.len() >= MIN_NAME && word.bytes().all(word_byte) && !word.as_bytes()[0].is_ascii_digit()
}

/// The word `name` became where `file` stopped defining it: a removed line
/// that uses `name`, and an added line equal to it, whitespace trimmed, once
/// `name` is replaced by a word that file's new version defines (`now`).
/// Otherwise such a line in another file of the same `diff` that defines
/// the word, as when `price()` in cart.js becomes `unitPrice()` in a new
/// pricing.js.
fn renamed_to(diff: &[FileDiff], file: &FileDiff, name: &str, now: &[String]) -> Option<String> {
    became(&file.removed, &file.added, name, |to| {
        now.iter().any(|d| d == to)
    })
    .or_else(|| {
        diff.iter()
            .filter(|f| f.path != file.path)
            .find_map(|other| {
                let source = other.new_source.as_deref()?;
                became(&file.removed, &other.added, name, |to| {
                    defined(&other.path, source).iter().any(|(d, _)| d == to)
                })
            })
    })
}

/// A word `to` that `defines`, for which one of the `added` lines equals one
/// of the `removed` lines that use `name` with `name` replaced by `to`.
fn became(
    removed: &[(u32, String)],
    added: &[(u32, String)],
    name: &str,
    defines: impl Fn(&str) -> bool,
) -> Option<String> {
    for (_, old) in removed {
        let old = old.trim();
        let Some(at) = word_starts(old, name).next() else {
            continue;
        };
        for (_, new) in added {
            let new = new.trim();
            let Some(rest) = new.strip_prefix(&old[..at]) else {
                continue;
            };
            let to = &rest[..rest.bytes().take_while(|&b| word_byte(b)).count()];
            if to != name && replace_words(old, name, to) == new && defines(to) {
                return Some(to.to_string());
            }
        }
    }
    None
}

/// The files of `diff`, shown whole, that say `name` only in strings and
/// comments, as pathspecs that leave them out of a grep; `None` when one of
/// them uses it as code.
// ponytail: only the files the branch changed are read, so a file it left
// alone that imports another export of price.js still counts as using
// `price`; read the files the grep finds too if real renames slip by.
fn said_only(diff: &[FileDiff], name: &str) -> Option<Vec<String>> {
    let mut files = Vec::new();
    for file in diff {
        let Some(source) = file.new_source.as_deref().filter(|s| has_word(s, name)) else {
            continue;
        };
        if uses_name(Path::new(&file.path), source, name) != Some(false) {
            return None;
        }
        files.push(format!(":(exclude,literal){}", file.path));
    }
    Some(files)
}

/// The lines an input added where its code refers to `name`, as the file is
/// on that branch: not a word in a string, a comment, docs or a file with no
/// grammar here, an object key, nor `a.name` unless `name` was a `member`.
/// Not in a file that binds `name` itself, a local variable or a parameter
/// included, whose code refers to its own.
fn uses_in(diff: &[FileDiff], name: &str, member: bool) -> Vec<Use> {
    let mut uses = Vec::new();
    for f in diff {
        let Some(source) = f.new_source.as_deref() else {
            continue;
        };
        if !f.added.iter().any(|(_, l)| has_word(l, name)) {
            continue;
        }
        let Some(said) = name_use(Path::new(&f.path), source, name, member) else {
            continue;
        };
        if said.binds {
            continue;
        }
        let added: BTreeSet<u32> = f.added.iter().map(|(n, _)| *n).collect();
        uses.extend(
            said.lines
                .iter()
                .filter_map(|&n| u32::try_from(n).ok().filter(|n| added.contains(n)))
                .map(|line| Use {
                    path: f.path.clone(),
                    line,
                }),
        );
    }
    uses
}

/// Whether the failing output says `name` where it says what went wrong, or
/// the place of one of `uses`, as a stack frame or a compiler gives it. An
/// error, an assertion, a place with what is wrong there or a test that
/// failed says what went wrong (`rank` 2 or better); a count of tests, a
/// diff of values or a test's title does not, nor a word in a path, such as
/// `price.js` or `src\price.js`.
fn confirms(failing: &[&[String]], name: &str, uses: &[Use]) -> bool {
    failing.iter().flat_map(|tail| tail.iter()).any(|line| {
        let line = line.trim();
        (says_outside_paths(line, name) && rank(line).is_some_and(|r| r <= 2)) || {
            let line = line.replace('\\', "/");
            uses.iter().any(|u| names_place(&line, u))
        }
    })
}

/// Extensions of the files a check prints the path of; `price.test.js` and
/// `price.js:6` are paths, `price.map` is not.
const EXTENSIONS: &[&str] = &[
    "js", "jsx", "mjs", "cjs", "ts", "tsx", "mts", "cts", "py", "pyi", "go", "rs", "rb", "java",
    "kt", "c", "h", "cc", "cpp", "cxx", "hpp", "cs", "php", "swift", "json", "md", "txt", "yml",
    "yaml", "toml", "test", "spec",
];

/// Whether `line` has `word` as a whole word that is not part of a path: not
/// after a `/` or `\`, nor before a file's extension.
fn says_outside_paths(line: &str, word: &str) -> bool {
    word_starts(line, word).any(|i| {
        let after = &line[i + word.len()..];
        let ext = after.strip_prefix('.').map(|rest| {
            let end = rest.bytes().take_while(|&b| word_byte(b)).count();
            &rest[..end]
        });
        !line[..i].ends_with(['/', '\\']) && !ext.is_some_and(|e| EXTENSIONS.contains(&e))
    })
}

/// `src/a.js:4` in `line`, as a stack trace or a compiler prints a place:
/// not the end of a longer name, nor the start of a longer number.
fn names_place(line: &str, at: &Use) -> bool {
    let place = format!("{}:{}", at.path, at.line);
    line.match_indices(&place).any(|(i, _)| {
        let before = line[..i].chars().next_back();
        let after = line[i + place.len()..].chars().next();
        !before.is_some_and(|c| c.is_alphanumeric() || "_-.".contains(c))
            && !after.is_some_and(|c| c.is_ascii_digit())
    })
}

/// A letter, digit or `_`: what `git grep -w` counts as part of a word.
fn word_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// The words of `line`, as `git grep -w` splits it.
fn words_of(line: &str) -> impl Iterator<Item = &str> {
    line.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .filter(|w| !w.is_empty())
}

/// Where `word`, itself a plain word, starts in `line` as a whole word.
fn word_starts<'a>(line: &'a str, word: &'a str) -> impl Iterator<Item = usize> + 'a {
    let bytes = line.as_bytes();
    line.match_indices(word).map(|(i, _)| i).filter(move |&i| {
        !(i > 0 && word_byte(bytes[i - 1]))
            && !bytes.get(i + word.len()).is_some_and(|&b| word_byte(b))
    })
}

fn has_word(line: &str, word: &str) -> bool {
    word_starts(line, word).next().is_some()
}

/// `line` with every whole word `from` replaced by `to`.
fn replace_words(line: &str, from: &str, to: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut last = 0;
    for i in word_starts(line, from) {
        out.push_str(&line[last..i]);
        out.push_str(to);
        last = i + from.len();
    }
    out.push_str(&line[last..]);
    out
}

/// `git diff from to`, file by file, in the same shape whatever the user's
/// config says: no renames, colors, external diff or text conversion, and
/// the usual `a/` and `b/` prefixes. Its context is as many lines as a file
/// to parse may have bytes, so the diff shows each such file whole, in one
/// hunk, and the sides to parse cost no read of their own.
fn diff_args(from: &str, to: &str) -> Vec<String> {
    let context = format!("-U{MAX_FILE_BYTES}");
    [
        "diff",
        &context,
        "--no-renames",
        "--no-color",
        "--no-ext-diff",
        "--no-textconv",
        "--submodule=short",
        "--src-prefix=a/",
        "--dst-prefix=b/",
        from,
        to,
        "--",
    ]
    .map(str::to_string)
    .to_vec()
}

fn parse_diff(text: &str) -> Vec<FileDiff> {
    let mut files: Vec<FileDiff> = Vec::new();
    let (mut old, mut new, mut hunks) = (0u32, 0u32, 0);
    for line in text.lines() {
        // A file with Windows line endings keeps its `\r` in the diff.
        let line = line.strip_suffix('\r').unwrap_or(line);
        if line.starts_with("diff --git ") {
            files.push(FileDiff::default());
            hunks = 0;
            continue;
        }
        let Some(file) = files.last_mut() else {
            continue;
        };
        if let Some(header) = line.strip_prefix("@@ ") {
            // `@@ -3,2 +3 @@`: where each side's lines start.
            let mut sides = header.split_whitespace();
            let mut start = || -> u32 {
                let side = sides.next().unwrap_or_default();
                let first = side.get(1..).unwrap_or_default().split(',').next();
                first.and_then(|n| n.parse().ok()).unwrap_or(0)
            };
            (old, new) = (start(), start());
            hunks += 1;
            // One hunk from the first line shows a side whole; a second one
            // means it did not.
            let whole = hunks == 1 && parses(Path::new(&file.path));
            file.old_source = (whole && file.before && old == 1).then(String::new);
            file.new_source = (whole && file.after && new == 1).then(String::new);
        } else if hunks == 0 {
            if let Some(p) = line.strip_prefix("--- ") {
                file.before = p != "/dev/null";
                if file.before {
                    file.path = header_path(p, "a/").unwrap_or_default();
                }
            } else if let Some(p) = line.strip_prefix("+++ ") {
                file.after = p != "/dev/null";
                if file.after {
                    file.path = header_path(p, "b/").unwrap_or_default();
                }
            }
        } else if let Some(text) = line.strip_prefix('+') {
            file.added.push((new, text.to_string()));
            keep(&mut file.new_source, text);
            new += 1;
        } else if let Some(text) = line.strip_prefix('-') {
            file.removed.push((old, text.to_string()));
            keep(&mut file.old_source, text);
            old += 1;
        } else if let Some(text) = line.strip_prefix(' ').or(line.is_empty().then_some("")) {
            // diff.suppressBlankEmpty leaves an empty line of context bare.
            keep(&mut file.old_source, text);
            keep(&mut file.new_source, text);
            old += 1;
            new += 1;
        }
    }
    // Binary files and mode changes have no lines; a quoted path is skipped.
    files.retain(|f| !f.path.is_empty());
    files
}

/// Adds `line` to a side being rebuilt, which is dropped once it is too big
/// to parse.
fn keep(side: &mut Option<String>, line: &str) {
    if let Some(text) = side {
        if text.len() + line.len() < MAX_FILE_BYTES {
            text.push_str(line);
            text.push('\n');
        } else {
            *side = None;
        }
    }
}

/// The path in a `---` or `+++` line, after `prefix`. `None` for one git
/// quoted (it has a `"`, a `\` or a control character), which is left out.
fn header_path(header: &str, prefix: &str) -> Option<String> {
    // Git ends a path that has a space with a tab.
    let header = header.strip_suffix('\t').unwrap_or(header);
    header.strip_prefix(prefix).map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{branch, git, repo, write, CART};

    /// `find` for `branches`, cut from main and merged onto it in that order,
    /// after a combined run that printed `output`.
    fn cause(d: &Path, branches: &[&str], output: &[&str]) -> Option<Cause> {
        cause_by(d, branches, output, None)
    }

    /// `cause`, the search stopping at `deadline`.
    fn cause_by(
        d: &Path,
        branches: &[&str],
        output: &[&str],
        deadline: Option<Instant>,
    ) -> Option<Cause> {
        let base = git(d, &["rev-parse", "main"]);
        let commits: Vec<String> = branches.iter().map(|b| git(d, &["rev-parse", b])).collect();
        git(d, &["checkout", "-q", "--detach", "main"]);
        let mut merge = vec!["merge", "-q", "--no-edit"];
        merge.extend(branches);
        git(d, &merge);
        let combined = git(d, &["rev-parse", "HEAD"]);
        git(d, &["checkout", "-q", "main"]);
        let merged: Vec<Merged> = commits
            .iter()
            .enumerate()
            .map(|(index, commit)| Merged {
                index,
                merge_base: &base,
                commit,
            })
            .collect();
        let tail: Vec<String> = output.iter().map(|l| l.to_string()).collect();
        find(&Git::user(d), &merged, &combined, &[&tail], deadline)
    }

    const CALLER: &str =
        "const { price } = require('./cart.js');\nmodule.exports = () => price({ cost: 2 });\n";

    fn uses(lines: &[u32]) -> Vec<Use> {
        lines
            .iter()
            .map(|&line| Use {
                path: "src/extra.js".into(),
                line,
            })
            .collect()
    }

    #[test]
    fn a_rename_the_other_branch_did_not_follow_is_the_cause() {
        let dir = repo();
        let d = dir.path();
        let renamed = CART.replace("price", "unitPrice");
        branch(d, "main", "agent/rename", &[("src/cart.js", &renamed)]);
        branch(d, "main", "agent/caller", &[("src/extra.js", CALLER)]);
        let failed = ["TypeError: price is not a function"];
        let found = cause(d, &["agent/rename", "agent/caller"], &failed);
        assert_eq!(
            found,
            Some(Cause {
                by: 0,
                user: 1,
                name: "price".into(),
                renamed_to: Some("unitPrice".into()),
                uses: uses(&[1, 2]),
                confirmed: true,
                also: vec![],
            })
        );
        // The positions follow the inputs' order.
        let found = cause(d, &["agent/caller", "agent/rename"], &failed).unwrap();
        assert_eq!((found.by, found.user), (1, 0));
    }

    /// Past the deadline the search starts no git call, and finds nothing.
    #[test]
    fn no_cause_is_looked_for_past_the_deadline() {
        let dir = repo();
        let d = dir.path();
        let renamed = CART.replace("price", "unitPrice");
        branch(d, "main", "agent/rename", &[("src/cart.js", &renamed)]);
        branch(d, "main", "agent/caller", &[("src/extra.js", CALLER)]);
        let failed = ["TypeError: price is not a function"];
        let both = ["agent/rename", "agent/caller"];
        assert_eq!(cause_by(d, &both, &failed, Some(Instant::now())), None);
        let later = Instant::now() + std::time::Duration::from_secs(600);
        assert!(cause_by(d, &both, &failed, Some(later)).is_some());
    }

    /// Two branches use the name a third renamed: the cause names one, and
    /// keeps the other, in either order.
    #[test]
    fn every_branch_that_uses_the_name_stays_with_the_cause() {
        let dir = repo();
        let d = dir.path();
        let renamed = CART.replace("price", "unitPrice");
        branch(d, "main", "agent/rename", &[("src/cart.js", &renamed)]);
        branch(d, "main", "agent/caller", &[("src/extra.js", CALLER)]);
        let tax = CALLER.replace("cost: 2", "cost: 3");
        branch(d, "main", "agent/tax", &[("src/tax.js", &tax)]);
        let failed = ["TypeError: price is not a function"];
        let found = cause(d, &["agent/rename", "agent/caller", "agent/tax"], &failed).unwrap();
        assert_eq!((found.by, found.user, found.also), (0, 1, vec![2]));
        let found = cause(d, &["agent/tax", "agent/caller", "agent/rename"], &failed).unwrap();
        assert_eq!((found.by, found.user, found.also), (2, 0, vec![1]));
    }

    /// `price()` in cart.js becomes `unitPrice()` in a new pricing.js: a
    /// rename to a name another file defines now, not a removal. A move of
    /// the whole file renames nothing, and is no cause.
    #[test]
    fn a_rename_into_another_file_is_a_rename() {
        let dir = repo();
        let d = dir.path();
        let pricing = "function unitPrice(item) {\n  return item.cost;\n}\n\nmodule.exports = { unitPrice };\n";
        let cart = CART
            .replace("function price(item) {\n  return item.cost;\n}\n\n", "")
            .replace("price(i)", "unitPrice(i)")
            .replace("{ price, total, label }", "{ total, label }");
        let cart = format!("const {{ unitPrice }} = require('./pricing.js');\n\n{cart}");
        branch(
            d,
            "main",
            "agent/across",
            &[("src/cart.js", &cart), ("src/pricing.js", pricing)],
        );
        branch(d, "main", "agent/caller", &[("src/extra.js", CALLER)]);
        let failed = ["TypeError: price is not a function"];
        let found = cause(d, &["agent/across", "agent/caller"], &failed).unwrap();
        assert_eq!(
            (found.name.as_str(), found.renamed_to.as_deref()),
            ("price", Some("unitPrice"))
        );
        git(d, &["checkout", "-q", "-b", "agent/move", "main"]);
        std::fs::create_dir_all(d.join("src/lib")).unwrap();
        git(d, &["mv", "src/cart.js", "src/lib/cart.js"]);
        git(d, &["commit", "-q", "-m", "move"]);
        git(d, &["checkout", "-q", "main"]);
        assert_eq!(cause(d, &["agent/move", "agent/caller"], &failed), None);
    }

    #[test]
    fn a_removed_function_another_branch_still_calls_is_the_cause() {
        let dir = repo();
        let d = dir.path();
        let removed = CART
            .replace("function price(item) {\n  return item.cost;\n}\n\n", "")
            .replace("sum += price(i)", "sum += i.cost")
            .replace("{ price, total, label }", "{ total, label }");
        branch(d, "main", "agent/cleanup", &[("src/cart.js", &removed)]);
        branch(d, "main", "agent/caller", &[("src/extra.js", CALLER)]);
        let found = cause(
            d,
            &["agent/cleanup", "agent/caller"],
            &["TypeError: price is not a function"],
        )
        .unwrap();
        assert_eq!(found.name, "price");
        assert_eq!(found.renamed_to, None);
        assert!(found.confirmed);
    }

    /// A caller that brings its own `price` does not use the one the other
    /// branch renamed.
    #[test]
    fn a_name_the_user_branch_defines_itself_is_no_cause() {
        let dir = repo();
        let d = dir.path();
        let renamed = CART.replace("price", "unitPrice");
        branch(d, "main", "agent/rename", &[("src/cart.js", &renamed)]);
        let own = "function price(item) {\n  return item.cost * 2;\n}\nmodule.exports = () => price({ cost: 2 });\n";
        branch(d, "main", "agent/caller", &[("src/extra.js", own)]);
        let found = cause(d, &["agent/rename", "agent/caller"], &["price"]);
        assert_eq!(found, None);
        // In a form that is no symbol, and imported by its own test.
        let test = "const assert = require('node:assert');\nconst { price } = require('../src/tax.js');\nassert.strictEqual(price([{ cost: 10 }]), 11);\n";
        for (n, tax) in [
            "const { total } = require('./cart.js');\nconst memo = (f) => f;\nconst price = memo((items) => Math.round(total(items) * 1.1));\nmodule.exports = { price };\n",
            "const { total } = require('./cart.js');\nexports.price = (items) => Math.round(total(items) * 1.1);\n",
        ]
        .into_iter()
        .enumerate()
        {
            let name = format!("agent/tax{n}");
            branch(
                d,
                "main",
                &name,
                &[("src/tax.js", tax), ("test/tax.test.js", test)],
            );
            let failed = ["AssertionError [ERR_ASSERTION]: 1100 !== 11"];
            assert_eq!(cause(d, &["agent/rename", &name], &failed), None, "{tax}");
        }
    }

    #[test]
    fn a_language_without_a_grammar_gives_no_cause() {
        let dir = repo();
        let d = dir.path();
        git(d, &["rm", "-q", "src/cart.js"]);
        write(
            d,
            &[(
                "src/cart.lua",
                "function price(item)\n  return item.cost\nend\n",
            )],
        );
        git(d, &["add", "-A"]);
        git(d, &["commit", "-q", "-m", "lua"]);
        let renamed = "function unitPrice(item)\n  return item.cost\nend\n";
        branch(d, "main", "agent/rename", &[("src/cart.lua", renamed)]);
        branch(
            d,
            "main",
            "agent/caller",
            &[("src/extra.lua", "return price({ cost = 2 })\n")],
        );
        let failed = ["attempt to call a nil value (global 'price')"];
        assert_eq!(cause(d, &["agent/rename", "agent/caller"], &failed), None);
    }

    /// The renaming branch still says `price` elsewhere, so it is not gone:
    /// the same test keeps common words such as `get` out.
    #[test]
    fn a_name_the_renaming_branch_still_mentions_is_no_cause() {
        let dir = repo();
        let d = dir.path();
        let renamed = CART.replace("price", "unitPrice");
        branch(
            d,
            "main",
            "agent/rename",
            &[
                ("src/cart.js", &renamed),
                ("docs/cart.md", "Call price() for one item.\n"),
            ],
        );
        branch(d, "main", "agent/caller", &[("src/extra.js", CALLER)]);
        let failed = ["TypeError: price is not a function"];
        assert_eq!(cause(d, &["agent/rename", "agent/caller"], &failed), None);
    }

    /// Renaming `price` in price.js leaves `require('./price.js')` behind, and
    /// often a comment: a file's name and prose, not a use of the function.
    /// A call left behind is one, and so is the word in docs, which no grammar
    /// here reads.
    #[test]
    fn a_name_left_only_in_strings_and_comments_is_gone() {
        let dir = repo();
        let d = dir.path();
        git(d, &["rm", "-q", "src/cart.js"]);
        let price =
            "function price(item) {\n  return item.cost;\n}\n\nmodule.exports = { price };\n";
        let total = "const { price } = require('./price.js');\n\nfunction total(items) {\n  return items.reduce((sum, i) => sum + price(i), 0);\n}\n\nmodule.exports = { total };\n";
        write(d, &[("src/price.js", price), ("src/total.js", total)]);
        git(d, &["add", "-A"]);
        git(d, &["commit", "-q", "-m", "price.js"]);
        let cost = price.replace("price", "cost");
        let followed = total
            .replace("{ price }", "{ cost }")
            .replace("price(i)", "cost(i)");
        let followed = format!("// cost was price\n{followed}");
        branch(
            d,
            "main",
            "agent/rename",
            &[("src/price.js", &cost), ("src/total.js", &followed)],
        );
        branch(d, "main", "agent/half", &[("src/price.js", &cost)]);
        let docs = [
            ("src/price.js", cost.as_str()),
            ("src/total.js", &followed),
            ("docs/price.md", "Call price() for one item.\n"),
        ];
        branch(d, "main", "agent/docs", &docs);
        let caller = "const { price } = require('./price.js');\nmodule.exports = () => price({ cost: 2 });\n";
        branch(d, "main", "agent/caller", &[("src/extra.js", caller)]);
        let failed = ["TypeError: price is not a function"];
        let found = cause(d, &["agent/rename", "agent/caller"], &failed).unwrap();
        assert_eq!(
            (found.name.as_str(), found.renamed_to.as_deref()),
            ("price", Some("cost"))
        );
        assert_eq!(found.uses, uses(&[1, 2]));
        // total.js still calls price on one; docs still say it on the other.
        for still in ["agent/half", "agent/docs"] {
            assert_eq!(cause(d, &[still, "agent/caller"], &failed), None, "{still}");
        }
    }

    /// A word in a new line is no use: the reviewers' repros, where the
    /// failure has nothing to do with the rename, and none is a cause.
    #[test]
    fn a_word_that_is_not_code_referring_to_the_name_is_no_use() {
        let dir = repo();
        let d = dir.path();
        // `total` renamed to `sum`; the other branch's `total` is a local.
        branch(
            d,
            "main",
            "agent/cents",
            &[("src/cart.js", &CART.replace("total", "sum"))],
        );
        let receipt = "const { price } = require('./cart.js');\n\nfunction receipt(items) {\n  let total = 0;\n  for (const i of items) total += price(i);\n  return 'Due: ' + total;\n}\n\nmodule.exports = { receipt };\n";
        branch(d, "main", "agent/receipt", &[("src/receipt.js", receipt)]);
        let jest = [
            "FAIL test/receipt.test.js",
            "    Expected: \"Due: 5\"",
            "    Received: \"Due: 500\"",
            "Test Suites: 1 failed, 1 passed, 2 total",
            "Tests:       1 failed, 1 passed, 2 total",
            "Snapshots:   0 total",
        ];
        assert_eq!(cause(d, &["agent/cents", "agent/receipt"], &jest), None);

        let renamed = CART.replace("price", "unitPrice");
        branch(d, "main", "agent/rename", &[("src/cart.js", &renamed)]);
        // An object key, and a comment and a test's title that say the word.
        let key = "const { total } = require('./cart.js');\n\nfunction quote(items) {\n  return { items: items.length, price: total(items) };\n}\n\nmodule.exports = { quote };\n";
        let key_test = "const assert = require('node:assert');\nconst { quote } = require('../src/quote.js');\nassert.deepStrictEqual(quote([{ cost: 2 }]), { items: 1, price: 2 });\n";
        branch(
            d,
            "main",
            "agent/key",
            &[("src/quote.js", key), ("test/quote.test.js", key_test)],
        );
        let said = "const { total } = require('./cart.js');\n\n// The quote's price is the cart total.\nfunction quote(items) {\n  return { items: items.length, amount: total(items) };\n}\n\nmodule.exports = { quote };\n";
        let titled = "const test = require('node:test');\nconst { quote } = require('../src/quote.js');\ntest('quote price is the cart total', () => quote([]));\n";
        branch(
            d,
            "main",
            "agent/said",
            &[("src/quote.js", said), ("test/quote.test.js", titled)],
        );
        let node = [
            "✖ quote price is the cart total (4.1924ms)",
            "  AssertionError [ERR_ASSERTION]: Expected values to be strictly deep-equal:",
            "  +   price: 500",
            "  -   price: 5",
        ];
        for user in ["agent/key", "agent/said"] {
            assert_eq!(cause(d, &["agent/rename", user], &node), None, "{user}");
        }

        // `price` renamed in price.js; the other branch only requires the
        // file, and the error names it only in a path.
        git(d, &["rm", "-q", "src/cart.js"]);
        let price = "function price(item) {\n  return item.cost;\n}\n\nfunction total(items) {\n  return items.reduce((sum, i) => sum + price(i), 0);\n}\n\nmodule.exports = { price, total };\n";
        write(d, &[("src/price.js", price)]);
        git(d, &["add", "-A"]);
        git(d, &["commit", "-q", "-m", "price.js"]);
        let strict = price.replace("price", "unitPrice").replace(
            "{\n  return items",
            "{\n  if (!items.length) throw new Error('empty cart');\n  return items",
        );
        branch(d, "main", "agent/strict", &[("src/price.js", &strict)]);
        let report = "const { total } = require('./price.js');\n\nfunction report(orders) {\n  return orders.map((items) => total(items));\n}\n\nmodule.exports = { report };\n";
        branch(d, "main", "agent/report", &[("src/report.js", report)]);
        let thrown = [
            "  Error: empty cart",
            r"      at total (C:\work\shop\src\price.js:6:33)",
        ];
        assert_eq!(cause(d, &["agent/strict", "agent/report"], &thrown), None);
    }

    /// README and docs say the word in prose, which no grammar here reads:
    /// the uses are the code's, and a branch with only docs is no user.
    #[test]
    fn prose_is_no_use() {
        let dir = repo();
        let d = dir.path();
        let renamed = CART.replace("price", "unitPrice");
        branch(d, "main", "agent/rename", &[("src/cart.js", &renamed)]);
        let readme = "# Shop\n\nDiscounts take ten percent off the item price.\n";
        branch(
            d,
            "main",
            "agent/discount",
            &[("README.md", readme), ("src/extra.js", CALLER)],
        );
        branch(
            d,
            "main",
            "agent/docs",
            &[("docs/pricing.md", "Every item has a price.\nThe price is in dollars.\nA total is the sum of each price.\n")],
        );
        let failed = ["TypeError: price is not a function"];
        let found = cause(d, &["agent/rename", "agent/discount"], &failed).unwrap();
        assert_eq!(found.uses, uses(&[1, 2]));
        let three = ["agent/rename", "agent/discount", "agent/docs"];
        let found = cause(d, &three, &failed).unwrap();
        assert_eq!((found.by, found.user), (0, 1));
        assert_eq!(cause(d, &["agent/rename", "agent/docs"], &failed), None);
    }

    /// The failing output confirms a name it gives as what went wrong, not
    /// one in a count, a diff of values, a test's title or a path; and the
    /// place of a use, as a stack frame or a compiler names it.
    #[test]
    fn only_what_went_wrong_confirms_a_name() {
        let at = [Use {
            path: "src/discount.js".into(),
            line: 4,
        }];
        let confirmed = |line: &str, name: &str| confirms(&[&[line.to_string()]], name, &at);
        for line in [
            "TypeError: price is not a function",
            "  TypeError [Error]: price is not a function",
            "FAIL discount.test.js: price is not a function",
            "E       NameError: name 'price' is not defined",
            ".\\discount.go:3:40: undefined: price",
            "AssertionError: price(1) == 2",
            "TypeError: cart.price is not a function",
        ] {
            assert!(confirmed(line, "price"), "{line}");
        }
        for line in [
            "Tests:       1 failed, 1 passed, 2 total",
            "Snapshots:   0 total",
            "  +   price: 500",
            "✖ quote price is the cart total (4.1924ms)",
            r"      at total (C:\work\src\price.js:6:33)",
            "Error: Cannot find module './price.js'",
            "FAIL test/price.test.js",
            "Error: empty cart",
        ] {
            assert!(!confirmed(line, "price"), "{line}");
            assert!(!confirmed(line, "total"), "{line}");
        }
        for frame in [
            "    at discounted (/tmp/run/src/discount.js:4:10)",
            r"    at discounted (C:\run\src\discount.js:4:10)",
            "src/discount.js:4: TypeError",
        ] {
            assert!(confirmed(frame, "unrelated"), "{frame}");
        }
    }

    #[test]
    fn output_that_names_neither_the_name_nor_a_use_leaves_it_unconfirmed() {
        let dir = repo();
        let d = dir.path();
        let renamed = CART.replace("price", "unitPrice");
        branch(d, "main", "agent/rename", &[("src/cart.js", &renamed)]);
        branch(d, "main", "agent/caller", &[("src/extra.js", CALLER)]);
        let branches = ["agent/rename", "agent/caller"];
        let found = cause(d, &branches, &["Error: Cannot find module 'left-pad'"]).unwrap();
        assert_eq!((found.name.as_str(), found.confirmed), ("price", false));
        // A stack frame at one of the uses confirms it, in Windows spelling too.
        for frame in [
            "    at module.exports (/tmp/run/src/extra.js:2:24)",
            r"    at module.exports (C:\run\src\extra.js:2:24)",
        ] {
            assert!(cause(d, &branches, &[frame]).unwrap().confirmed, "{frame}");
        }
        // Not a longer line number, nor a file whose name ends the same way.
        for frame in [
            "at x (/tmp/src/extra.js:21:3)",
            "at x (/tmp/old-src/extra.js:2:1)",
        ] {
            assert!(!cause(d, &branches, &[frame]).unwrap().confirmed, "{frame}");
        }
    }

    #[test]
    fn windows_line_endings_change_nothing() {
        let dir = repo();
        let d = dir.path();
        let crlf = |text: &str| text.replace('\n', "\r\n");
        write(d, &[("src/cart.js", &crlf(CART))]);
        git(d, &["commit", "-q", "-am", "crlf"]);
        let renamed = crlf(&CART.replace("price", "unitPrice"));
        branch(d, "main", "agent/rename", &[("src/cart.js", &renamed)]);
        branch(
            d,
            "main",
            "agent/caller",
            &[("src/extra.js", &crlf(CALLER))],
        );
        let found = cause(
            d,
            &["agent/rename", "agent/caller"],
            &["TypeError: price is not a function"],
        )
        .unwrap();
        assert_eq!(found.renamed_to.as_deref(), Some("unitPrice"));
        assert_eq!(found.uses, uses(&[1, 2]));
    }

    #[test]
    fn a_diff_reads_the_same_whatever_its_paths_and_line_endings() {
        let text = "diff --git a/src/a b.js b/src/a b.js\nindex 1..2 100644\n--- a/src/a b.js\t\n+++ b/src/a b.js\t\n@@ -3,2 +3 @@ fn x\n-old one\r\n-old two\r\n+new\r\n@@ -9,0 +9,2 @@\n+p\n+q\n\\ No newline at end of file\ndiff --git a/gone.js b/gone.js\ndeleted file mode 100644\n--- a/gone.js\n+++ /dev/null\n@@ -1 +0,0 @@\n-x\ndiff --git \"a/q\\\"x.js\" \"b/q\\\"x.js\"\n--- \"a/q\\\"x.js\"\n+++ \"b/q\\\"x.js\"\n@@ -1 +1 @@\n-a\n+b\ndiff --git a/i.png b/i.png\nBinary files a/i.png and b/i.png differ\n";
        let files = parse_diff(text);
        assert_eq!(files.len(), 2);
        let a = &files[0];
        assert_eq!(
            (a.path.as_str(), a.before, a.after),
            ("src/a b.js", true, true)
        );
        let removed: Vec<(u32, &str)> = a.removed.iter().map(|(n, l)| (*n, l.as_str())).collect();
        assert_eq!(removed, [(3, "old one"), (4, "old two")]);
        let added: Vec<(u32, &str)> = a.added.iter().map(|(n, l)| (*n, l.as_str())).collect();
        assert_eq!(added, [(3, "new"), (9, "p"), (10, "q")]);
        // Two hunks show neither side whole.
        assert!(a.old_source.is_none() && a.new_source.is_none());
        let gone = &files[1];
        assert_eq!(
            (gone.path.as_str(), gone.before, gone.after),
            ("gone.js", true, false)
        );
        assert_eq!(
            (gone.old_source.as_deref(), gone.new_source.as_deref()),
            (Some("x\n"), None)
        );
    }

    /// One hunk from the first line is the whole file on each side, an empty
    /// line of context left bare included; a side that starts later is not.
    #[test]
    fn a_whole_file_hunk_gives_both_sides() {
        let text = "diff --git a/a.js b/a.js\n--- a/a.js\n+++ b/a.js\n@@ -1,4 +1,4 @@\n-function price() {}\r\n+function unitPrice() {}\r\n \r\n\n module.exports = 1;\ndiff --git a/b.js b/b.js\n--- a/b.js\n+++ b/b.js\n@@ -2,1 +2,1 @@\n-x\n+y\n";
        let files = parse_diff(text);
        let a = &files[0];
        assert_eq!(
            a.old_source.as_deref(),
            Some("function price() {}\n\n\nmodule.exports = 1;\n")
        );
        assert_eq!(
            a.new_source.as_deref(),
            Some("function unitPrice() {}\n\n\nmodule.exports = 1;\n")
        );
        assert_eq!((a.removed[0].0, a.added[0].0), (1, 1));
        let b = &files[1];
        assert!(b.old_source.is_none() && b.new_source.is_none());
        assert_eq!((b.removed[0].0, b.added[0].0), (2, 2));
    }

    #[test]
    fn words_are_whole_words_as_git_grep_sees_them() {
        assert!(has_word("x = price(i)", "price"));
        assert!(!has_word("x = unitPrice(i) + prices", "price"));
        assert!(!has_word("x = price_2", "price"));
        assert_eq!(
            replace_words("price + price2 + price", "price", "cost"),
            "cost + price2 + cost"
        );
        assert_eq!(
            words_of("a.b(c_d, 1é2)").collect::<Vec<_>>(),
            ["a", "b", "c_d", "1", "2"]
        );
    }
}
