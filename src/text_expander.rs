#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct TextExpansionRule {
    #[serde(default = "default_rule_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub trigger: String,
    #[serde(default)]
    pub replacement: String,
}

fn default_rule_enabled() -> bool {
    true
}

impl Default for TextExpansionRule {
    fn default() -> Self {
        Self {
            enabled: true,
            trigger: String::new(),
            replacement: String::new(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TextExpansionConfig {
    pub enabled: bool,
    pub rules: Vec<TextExpansionRule>,
    pub app_blacklist: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextExpansionMatch {
    pub typed_trigger_chars: usize,
    pub replacement: String,
    pub cursor_back_chars: usize,
}

#[derive(Clone, Debug)]
pub struct TextExpansionEngine {
    rules: Vec<TextExpansionRule>,
    buffer: String,
    max_buffer_chars: usize,
}

impl Default for TextExpansionEngine {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl TextExpansionEngine {
    pub fn new(rules: Vec<TextExpansionRule>) -> Self {
        let max_trigger_chars = rules
            .iter()
            .filter(|rule| rule.enabled && !rule.trigger.is_empty())
            .map(|rule| rule.trigger.chars().count())
            .max()
            .unwrap_or(0);
        Self {
            rules,
            buffer: String::new(),
            max_buffer_chars: max_trigger_chars.max(64).min(128),
        }
    }

    pub fn set_rules(&mut self, rules: Vec<TextExpansionRule>) {
        *self = Self::new(rules);
    }

    pub fn reset(&mut self) {
        self.buffer.clear();
    }

    pub fn backspace(&mut self) {
        self.buffer.pop();
    }

    pub fn push_char(&mut self, ch: char) -> Option<TextExpansionMatch> {
        if ch.is_control() {
            self.reset();
            return None;
        }
        self.buffer.push(ch);
        self.trim_buffer();
        self.find_match().inspect(|_| self.reset())
    }

    fn trim_buffer(&mut self) {
        let len = self.buffer.chars().count();
        if len <= self.max_buffer_chars {
            return;
        }
        let keep_from = len - self.max_buffer_chars;
        let byte_idx = self
            .buffer
            .char_indices()
            .nth(keep_from)
            .map(|(idx, _)| idx)
            .unwrap_or(0);
        self.buffer.drain(..byte_idx);
    }

    fn find_match(&self) -> Option<TextExpansionMatch> {
        self.rules
            .iter()
            .filter(|rule| rule_usable(rule))
            .filter(|rule| self.buffer.ends_with(&rule.trigger))
            .filter(|rule| self.boundary_ok(&rule.trigger))
            .filter(|rule| !self.has_longer_pending_trigger(&rule.trigger))
            .max_by_key(|rule| rule.trigger.chars().count())
            .map(|rule| {
                let (replacement, cursor_back_chars) = prepare_replacement(&rule.replacement);
                TextExpansionMatch {
                    typed_trigger_chars: rule.trigger.chars().count(),
                    replacement,
                    cursor_back_chars,
                }
            })
    }

    fn has_longer_pending_trigger(&self, trigger: &str) -> bool {
        self.rules.iter().any(|rule| {
            rule_usable(rule)
                && rule.trigger.chars().count() > trigger.chars().count()
                && rule.trigger.starts_with(trigger)
        })
    }

    fn boundary_ok(&self, trigger: &str) -> bool {
        let Some(first) = trigger.chars().next() else {
            return false;
        };
        if !is_word_char(first) {
            return true;
        }
        let buffer_chars = self.buffer.chars().collect::<Vec<_>>();
        let trigger_len = trigger.chars().count();
        if buffer_chars.len() <= trigger_len {
            return true;
        }
        !is_word_char(buffer_chars[buffer_chars.len() - trigger_len - 1])
    }
}

pub fn rule_usable(rule: &TextExpansionRule) -> bool {
    rule.enabled
        && runtime_trigger_usable(&rule.trigger)
        && !prepare_replacement(&rule.replacement).0.is_empty()
}

fn runtime_trigger_usable(trigger: &str) -> bool {
    valid_trigger(trigger)
}

pub fn valid_trigger(trigger: &str) -> bool {
    let trimmed = trigger.trim();
    trimmed == trigger
        && trigger.chars().count() >= 2
        && matches!(trigger.chars().next(), Some(':') | Some(';'))
        && !trigger.chars().any(char::is_whitespace)
}

pub fn prepare_replacement(raw: &str) -> (String, usize) {
    let expanded = decode_text_escapes(raw);
    let marker = "$|$";
    if let Some(marker_byte_idx) = expanded.find(marker) {
        let before = expanded[..marker_byte_idx].to_owned();
        let after = expanded[marker_byte_idx + marker.len()..].to_owned();
        let cursor_back_chars = after.chars().count();
        (format!("{before}{after}"), cursor_back_chars)
    } else {
        (expanded, 0)
    }
}

fn decode_text_escapes(raw: &str) -> String {
    let mut output = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.peek().copied() {
                Some('n') => {
                    chars.next();
                    output.push('\n');
                }
                Some('t') => {
                    chars.next();
                    output.push('\t');
                }
                Some('r') => {
                    chars.next();
                    output.push('\r');
                }
                Some('\\') => {
                    chars.next();
                    output.push('\\');
                }
                _ => output.push(ch),
            }
        } else {
            output.push(ch);
        }
    }
    output
}

fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(trigger: &str, replacement: &str) -> TextExpansionRule {
        TextExpansionRule {
            enabled: true,
            trigger: trigger.to_owned(),
            replacement: replacement.to_owned(),
        }
    }

    #[test]
    fn expands_punctuation_trigger() {
        let mut engine = TextExpansionEngine::new(vec![rule(":addr", "Earth")]);
        let mut matched = None;
        for ch in ":addr".chars() {
            matched = engine.push_char(ch);
        }
        assert_eq!(
            matched,
            Some(TextExpansionMatch {
                typed_trigger_chars: 5,
                replacement: "Earth".to_owned(),
                cursor_back_chars: 0,
            })
        );
    }

    #[test]
    fn prefers_longest_trigger() {
        let mut engine = TextExpansionEngine::new(vec![rule(":a", "A"), rule(":addr", "Earth")]);
        let mut matched = None;
        for ch in ":addr".chars() {
            matched = engine.push_char(ch);
        }
        assert_eq!(matched.unwrap().replacement, "Earth");
    }

    #[test]
    fn ignores_triggers_without_prefix() {
        let mut engine = TextExpansionEngine::new(vec![rule("addr", "Earth")]);
        for ch in " addr".chars() {
            assert!(engine.push_char(ch).is_none());
        }
    }

    #[test]
    fn accepts_semicolon_prefix_trigger() {
        let mut engine = TextExpansionEngine::new(vec![rule(";addr", "Earth")]);
        let mut matched = None;
        for ch in ";addr".chars() {
            matched = engine.push_char(ch);
        }
        assert_eq!(matched.unwrap().replacement, "Earth");
    }

    #[test]
    fn backspace_updates_buffer() {
        let mut engine = TextExpansionEngine::new(vec![rule(":addr", "Earth")]);
        for ch in ":adx".chars() {
            assert!(engine.push_char(ch).is_none());
        }
        engine.backspace();
        assert!(engine.push_char('d').is_none());
        assert_eq!(engine.push_char('r').unwrap().replacement, "Earth");
    }

    #[test]
    fn prepares_multiline_and_cursor_marker() {
        let mut engine = TextExpansionEngine::new(vec![rule(":sig", "Hello\n$|$World")]);
        let mut matched = None;
        for ch in ":sig".chars() {
            matched = engine.push_char(ch);
        }
        assert_eq!(
            matched.unwrap(),
            TextExpansionMatch {
                typed_trigger_chars: 4,
                replacement: "Hello\nWorld".to_owned(),
                cursor_back_chars: 5,
            }
        );
    }
}
