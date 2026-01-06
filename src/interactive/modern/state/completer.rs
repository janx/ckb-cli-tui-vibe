use std::collections::HashSet;

pub struct TuiCompleter;

#[derive(Debug, Clone)]
pub struct Completion {
    pub display: String,
    pub replacement: String,
    pub is_required: bool,
}

impl TuiCompleter {
    pub fn new(_clap_app: &clap::App<'static>) -> Self {
        Self
    }

    pub fn get_completions(&self, input: &str, clap_app: &clap::App<'static>) -> Vec<Completion> {
        let args = match shell_words::split(input) {
            Ok(a) => a,
            Err(_) => return Vec::new(),
        };

        let current_word = if input.ends_with(' ') || input.is_empty() {
            ""
        } else {
            args.last().map(|s| s.as_str()).unwrap_or("")
        };

        let context_args: Vec<&str> = if input.ends_with(' ') || input.is_empty() {
            args.iter().map(|s| s.as_str()).collect()
        } else if args.len() > 1 {
            args[..args.len() - 1].iter().map(|s| s.as_str()).collect()
        } else {
            Vec::new()
        };

        let current_app = Self::find_subcommand(clap_app.clone(), context_args.iter().copied());
        let available = Self::get_context_completions(&current_app, &args);

        let word_lower = current_word.to_lowercase();
        if word_lower.is_empty() {
            available
        } else {
            available
                .into_iter()
                .filter(|c| {
                    c.replacement.to_lowercase().contains(&word_lower)
                        || fuzzy_match(&c.replacement.to_lowercase(), &word_lower)
                })
                .collect()
        }
    }

    fn find_subcommand<'a, 'b>(
        app: clap::App<'a>,
        mut args: impl Iterator<Item = &'b str>,
    ) -> clap::App<'a> {
        if let Some(name) = args.next() {
            for sub in app.get_subcommands() {
                if sub.get_name() == name || sub.get_all_aliases().any(|a| a == name) {
                    return Self::find_subcommand(sub.clone(), args);
                }
            }
        }
        app
    }

    fn get_context_completions(app: &clap::App<'static>, used_args: &[String]) -> Vec<Completion> {
        let used_set: HashSet<&str> = used_args.iter().map(|s| s.as_str()).collect();
        let mut completions = Vec::new();

        for sub in app.get_subcommands() {
            if !used_set.contains(sub.get_name()) {
                completions.push(Completion {
                    display: sub.get_name().to_string(),
                    replacement: sub.get_name().to_string(),
                    is_required: false,
                });
            }
        }

        for arg in app.get_arguments() {
            let long = arg.get_long().map(|s| format!("--{}", s));
            let short = arg.get_short().map(|c| format!("-{}", c));
            let is_required = arg.is_set(clap::ArgSettings::Required);
            let is_multiple = arg.is_set(clap::ArgSettings::MultipleValues);

            let already_used = long
                .as_ref()
                .map(|l| used_set.contains(l.as_str()))
                .unwrap_or(false)
                || short
                    .as_ref()
                    .map(|s| used_set.contains(s.as_str()))
                    .unwrap_or(false);

            if is_multiple || !already_used {
                if let Some(l) = long {
                    let display = if is_required {
                        format!("{}(*)", l)
                    } else {
                        l.clone()
                    };
                    completions.push(Completion {
                        display,
                        replacement: l,
                        is_required,
                    });
                }
                if let Some(s) = short {
                    let display = if is_required {
                        format!("{}(*)", s)
                    } else {
                        s.clone()
                    };
                    completions.push(Completion {
                        display,
                        replacement: s,
                        is_required,
                    });
                }
            }
        }

        completions
    }
}

fn fuzzy_match(text: &str, pattern: &str) -> bool {
    let mut pattern_chars = pattern.chars().peekable();

    for c in text.chars() {
        if pattern_chars.peek() == Some(&c) {
            pattern_chars.next();
        }
        if pattern_chars.peek().is_none() {
            return true;
        }
    }

    pattern_chars.peek().is_none()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_match() {
        assert!(fuzzy_match("wallet", "wlt"));
        assert!(fuzzy_match("wallet", "wal"));
        assert!(fuzzy_match("get_tip_header", "gth"));
        assert!(!fuzzy_match("wallet", "xyz"));
    }
}
