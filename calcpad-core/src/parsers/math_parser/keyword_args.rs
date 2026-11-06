//! Port of Parsers/MathParser/MathParser.KeywordArgs.cs.
use std::collections::VecDeque;
use std::rc::Rc;

use super::input::get_rpn;
use super::is_assignment;
use super::tokens::{Token, TokenKind, TokenRef, TokenTypes};
use super::MathParser;
use crate::exceptions;
use crate::math_parser_exception::CResult;

/// `ReadDefault(input)`: reads a default parameter expression up to `;` or `)`.
pub fn read_default(input: &mut VecDeque<TokenRef>) -> Vec<TokenRef> {
    let mut tokens = Vec::new();
    let mut depth = 0;
    while let Some(t) = input.front().cloned() {
        let tt = t.borrow().ttype;
        if depth == 0 && (tt == TokenTypes::Divisor || tt == TokenTypes::BracketRight) {
            break;
        }
        if tt == TokenTypes::BracketLeft || tt == TokenTypes::SquareBracketLeft {
            depth += 1;
        } else if tt == TokenTypes::BracketRight || tt == TokenTypes::SquareBracketRight {
            depth -= 1;
        }
        tokens.push(input.pop_front().unwrap());
    }
    get_rpn(&tokens)
}

fn is_keyword_argument(tokens: &[TokenRef], start: usize) -> bool {
    if start + 1 >= tokens.len() {
        return false;
    }
    let t = tokens[start].borrow();
    let n = tokens[start + 1].borrow();
    (t.is_variable_token() || t.ttype == TokenTypes::Unit)
        && n.ttype == TokenTypes::Operator
        && n.content == "="
        // `==` is a comparison, not a keyword argument
        && !(start + 2 < tokens.len() && tokens[start + 2].borrow().content == "=")
}

fn index_of_closing_bracket(tokens: &[TokenRef], open: usize) -> Option<usize> {
    let mut depth = 0;
    for (i, t) in tokens.iter().enumerate().skip(open) {
        let tt = t.borrow().ttype;
        if tt == TokenTypes::BracketLeft || tt == TokenTypes::SquareBracketLeft {
            depth += 1;
        } else if tt == TokenTypes::BracketRight || tt == TokenTypes::SquareBracketRight {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
    }
    None
}

fn argument_starts(tokens: &[TokenRef], start: usize, end: usize) -> Vec<usize> {
    let mut starts = vec![start];
    let mut depth = 0;
    for (i, t) in tokens.iter().enumerate().take(end).skip(start) {
        let tt = t.borrow().ttype;
        if tt == TokenTypes::BracketLeft || tt == TokenTypes::SquareBracketLeft {
            depth += 1;
        } else if tt == TokenTypes::BracketRight || tt == TokenTypes::SquareBracketRight {
            depth -= 1;
        } else if depth == 0 && tt == TokenTypes::Divisor {
            starts.push(i + 1);
        }
    }
    starts
}

/// `MathParser.FillDefaults`
pub fn fill_defaults(arguments: &mut [Option<String>], defaults: &[Option<String>]) -> CResult<()> {
    for i in 0..arguments.len() {
        if arguments[i].is_none() {
            match &defaults[i] {
                Some(d) => arguments[i] = Some(d.clone()),
                None => return Err(exceptions::InvalidNumberOfArguments()),
            }
        }
    }
    Ok(())
}

/// `MathParser.IndexOfTopLevel` on chars; returns a char index.
pub fn index_of_top_level(s: &[char], target: char) -> Option<usize> {
    let mut depth = 0i32;
    for (i, &c) in s.iter().enumerate() {
        if c == target && depth == 0 {
            return Some(i);
        }
        if c == '(' || c == '[' || c == '{' {
            depth += 1;
        } else if c == ')' || c == ']' || c == '}' {
            depth -= 1;
            if depth < 0 {
                return Some(i);
            }
        }
    }
    None
}

impl MathParser {
    /// `ResolveCallArguments`: "f(3; z = 4)" becomes supplied "3; 4" with map [0, 2].
    pub(crate) fn resolve_call_arguments(&self, input: Vec<TokenRef>) -> CResult<Vec<TokenRef>> {
        let has_call = input.iter().any(|t| {
            let t = t.borrow();
            t.ttype == TokenTypes::CustomFunction && t.index >= 0
        });
        if !has_call {
            return Ok(input);
        }
        let mut tokens = input.clone();
        let len = tokens.len();
        let mut is_resolved = false;
        let mut removed: Option<Vec<bool>> = None;
        let mut i = 0;
        while i + 2 < len {
            let (tt, index, content) = {
                let t = tokens[i].borrow();
                (t.ttype, t.index, t.content.clone())
            };
            if tt != TokenTypes::CustomFunction || index < 0 || tokens[i + 1].borrow().ttype != TokenTypes::BracketLeft
            {
                i += 1;
                continue;
            }
            let close = match index_of_closing_bracket(&tokens, i + 1) {
                None => break,
                Some(c) => c,
            };
            if i == 0 && close + 1 < len && is_assignment(&tokens[close + 1].borrow().content) || close == i + 2 {
                i += 1;
                continue;
            }
            let starts = argument_starts(&tokens, i + 2, close);
            let cf = self.functions.borrow().get(index).unwrap();
            let n = cf.parameter_count();
            let mut is_keyword = vec![false; starts.len()];
            let mut has_keyword = false;
            for k in 0..starts.len() {
                is_keyword[k] = is_keyword_argument(&tokens, starts[k]);
                has_keyword |= is_keyword[k];
            }
            if !has_keyword && starts.len() >= n {
                i += 1;
                continue;
            }
            let mut map = vec![0usize; starts.len()];
            let mut is_supplied = vec![false; n];
            let mut positional_count = 0usize;
            for k in 0..starts.len() {
                let idx;
                if is_keyword[k] {
                    let name = tokens[starts[k]].borrow().content.clone();
                    idx = match cf.index_of_parameter(&name) {
                        Some(x) => x,
                        None => return Err(exceptions::UnknownKeywordArgument(&name)),
                    };
                    if is_supplied[idx] {
                        return Err(exceptions::DuplicateArgument(&name));
                    }
                    let r = removed.get_or_insert_with(|| vec![false; len]);
                    r[starts[k]] = true;
                    r[starts[k] + 1] = true;
                } else if k > positional_count || positional_count >= n {
                    return Err(exceptions::InvalidNumberOfArguments());
                } else {
                    idx = positional_count;
                    positional_count += 1;
                }
                map[k] = idx;
                is_supplied[idx] = true;
            }
            {
                let defaults = cf.defaults.borrow();
                for p in 0..n {
                    let has_default = defaults.as_ref().is_some_and(|d| d.get(p).is_some_and(|x| x.is_some()));
                    if !is_supplied[p] && !has_default {
                        return Err(exceptions::InvalidNumberOfArguments());
                    }
                }
            }
            is_resolved = true;
            let mut ft = Token::function(content);
            ft.ttype = TokenTypes::CustomFunction;
            ft.index = index;
            ft.kind = TokenKind::Function {
                parameter_count: starts.len() as i32,
                argument_map: Some(Rc::new(map)),
                positional_count: positional_count as i32,
            };
            tokens[i] = ft.into_ref();
            i += 1;
        }
        if !is_resolved {
            return Ok(input);
        }
        let mut output = Vec::with_capacity(len);
        let assignment_index = self.assignment_index.get();
        for (i, t) in tokens.into_iter().enumerate() {
            let is_removed = removed.as_ref().is_some_and(|r| r[i]);
            if !is_removed {
                output.push(t);
            } else if i < assignment_index {
                self.assignment_index.set(self.assignment_index.get() - 1);
            }
        }
        Ok(output)
    }
}
