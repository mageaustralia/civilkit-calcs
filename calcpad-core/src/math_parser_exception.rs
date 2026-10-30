//! Port of MathParserException.cs.
use std::cell::RefCell;
use std::fmt;

use crate::netfmt::fmt as net_fmt;

/// Error raised by the math engine. `message` is the user visible text.
#[derive(Clone, Debug, PartialEq)]
pub struct MathParserException {
    pub message: String,
    /// Set for failures that in C# are not MathParserException (for example
    /// IndexOutOfRangeException). ExpressionParser reports those as "Unexpected error".
    pub unexpected: bool,
    /// The name of the entry in `messages.rs` the text was made from, for the messages that are raised with
    /// [`MathParserException::coded`]. `message` is always the English text; the code and arguments are what lets a host
    /// show the same message in another language (see [`message_codes`]).
    pub code: Option<&'static str>,
    /// The arguments the text was made with, in the order of the template's `{0}`, `{1}`.
    pub args: Vec<String>,
}

impl MathParserException {
    pub fn new(message: impl Into<String>) -> Self {
        MathParserException {
            message: message.into(),
            unexpected: false,
            code: None,
            args: Vec::new(),
        }
    }

    /// A message from the catalogue: `code` is the constant's name, `template` its text. The text is the same as
    /// `fmt(template, args)` has always produced; the code and arguments travel with it.
    pub fn coded(code: &'static str, template: &str, args: &[&str]) -> Self {
        let message = if args.is_empty() {
            template.to_string()
        } else {
            net_fmt(template, args)
        };
        message_codes::note(code, &message, args);
        MathParserException {
            message,
            unexpected: false,
            code: Some(code),
            args: args.iter().map(|a| a.to_string()).collect(),
        }
    }

    /// A non MathParserException failure (.NET runtime exception) with its message.
    pub fn unexpected(message: impl Into<String>) -> Self {
        MathParserException {
            message: message.into(),
            unexpected: true,
            code: None,
            args: Vec::new(),
        }
    }
}

impl fmt::Display for MathParserException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for MathParserException {}

pub type CResult<T> = Result<T, MathParserException>;

/// A record of the coded messages raised during one run, for a host that translates them.
///
/// The engine's text stays English. When a host switches collecting on, every message made with
/// [`MathParserException::coded`] (and the "Error in ..." frame around one) is noted with its code and arguments; the host
/// finds the text in the page it gets back and marks it up (calcpad-wasm), so the page can be shown in another language
/// without the engine knowing about languages. Collecting is off by default, so nothing changes for any other caller.
pub mod message_codes {
    use super::RefCell;

    /// One coded message: its final English text, the catalogue name and the arguments.
    #[derive(Clone, Debug, PartialEq)]
    pub struct Coded {
        pub text: String,
        pub code: &'static str,
        pub args: Vec<String>,
    }

    thread_local! {
        static LOG: RefCell<Option<Vec<Coded>>> = const { RefCell::new(None) };
    }

    /// Starts collecting (and forgets anything collected before).
    pub fn start() {
        LOG.with(|l| *l.borrow_mut() = Some(Vec::new()));
    }

    /// Stops collecting and returns what was collected.
    pub fn finish() -> Vec<Coded> {
        LOG.with(|l| l.borrow_mut().take()).unwrap_or_default()
    }

    /// `fmt(template, args)`, noted as a coded message: for text that is not an error of its own but frames one.
    pub fn format(code: &'static str, template: &str, args: &[&str]) -> String {
        let text = super::net_fmt(template, args);
        note(code, &text, args);
        text
    }

    /// Notes a message when collecting is on.
    pub fn note(code: &'static str, text: &str, args: &[&str]) {
        LOG.with(|l| {
            if let Some(v) = l.borrow_mut().as_mut() {
                if !v.iter().any(|c| {
                    c.text == text
                        && c.code == code
                        && c.args.len() == args.len()
                        && c.args.iter().zip(args).all(|(a, b)| a == b)
                }) {
                    v.push(Coded {
                        text: text.to_string(),
                        code,
                        args: args.iter().map(|a| a.to_string()).collect(),
                    });
                }
            }
        });
    }
}
