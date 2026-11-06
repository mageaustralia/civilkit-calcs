//! Port of Parsers/ExpressionParser/ExpressionParser.Condition.cs.
use crate::base_types::complex::Complex;
use crate::exceptions;
use crate::math_parser_exception::CResult;
use crate::netfmt;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Types {
    None,
    If,
    ElseIf,
    Else,
    EndIf,
    While,
}

#[derive(Clone, Copy)]
struct Item {
    value: bool,
    ttype: Types,
    /// Output and source line of the `#if` or `#while` that opened the block
    line: i32,
    source_line: i32,
}

/// `Keyword.End_If - Keyword.If`
pub const REMOVE_CONDITION_KEYWORD: i32 = 3;

pub struct Condition {
    count: usize,
    keyword: &'static str,
    keyword_length: usize,
    conditions: Vec<Item>,
    pub id: usize,
    pub is_unchecked: bool,
    pub is_found: bool,
}

impl Condition {
    pub fn new() -> Condition {
        let mut conditions = vec![
            Item {
                value: false,
                ttype: Types::None,
                line: -1,
                source_line: 0,
            };
            20
        ];
        conditions[0] = Item {
            value: true,
            ttype: Types::None,
            line: -1,
            source_line: 0,
        };
        Condition {
            count: 0,
            keyword: "",
            keyword_length: 0,
            conditions,
            id: 0,
            is_unchecked: false,
            is_found: false,
        }
    }

    fn item_type(&self) -> Types {
        self.conditions[self.id].ttype
    }

    pub fn is_satisfied(&self) -> bool {
        self.conditions[self.count].value
    }

    /// Where the innermost unclosed `#if` started, as (output line, source line).
    pub fn open_if(&self) -> Option<(i32, i32)> {
        (1..=self.id.min(self.conditions.len() - 1))
            .rev()
            .map(|i| &self.conditions[i])
            .find(|c| c.ttype != Types::While)
            .map(|c| (c.line, c.source_line))
    }

    pub fn keyword_length(&self) -> usize {
        self.keyword_length
    }

    pub fn is_loop(&self) -> bool {
        self.conditions[self.count].ttype == Types::While
    }

    fn ensure(&mut self, i: usize) -> CResult<()> {
        if i >= self.conditions.len() {
            return Err(crate::calculator::index_out_of_range());
        }
        Ok(())
    }

    fn add(&mut self, value: bool, line: i32, source_line: i32) -> CResult<()> {
        self.id += 1;
        self.ensure(self.id)?;
        self.conditions[self.id] = Item {
            value,
            ttype: Types::If,
            line,
            source_line,
        };
        if self.is_satisfied() {
            self.count += 1;
            self.is_found = false;
        }
        Ok(())
    }

    fn remove(&mut self) {
        if self.id == 0 {
            // C#: --Id would go negative; later indexing throws.
            self.id = usize::MAX;
            return;
        }
        self.id -= 1;
        if self.count > self.id {
            self.count -= 1;
            self.is_found = true;
        }
    }

    fn change(&mut self, value: bool, ttype: Types) {
        if self.id < self.conditions.len() {
            let Item { line, source_line, .. } = self.conditions[self.id];
            self.conditions[self.id] = Item {
                value,
                ttype,
                line,
                source_line,
            };
        }
    }

    pub fn set_condition(&mut self, index: i32, line: i32, source_line: i32) -> CResult<()> {
        if index < 0 || index >= 5 {
            if self.keyword_length > 0 {
                self.keyword_length = 0;
                self.keyword = "";
            }
            return Ok(());
        }
        let ttype = match index {
            0 => Types::If,
            1 => Types::ElseIf,
            2 => Types::Else,
            3 => Types::EndIf,
            _ => Types::While,
        };
        self.keyword = get_conditional_keyword(ttype);
        self.keyword_length = self.keyword.trim_end().chars().count();
        self.is_unchecked = ttype == Types::If || ttype == Types::ElseIf;
        if ttype > Types::If && ttype < Types::While && self.count == 0 {
            return Err(exceptions::ConditionNotInitialized());
        }
        if self.id >= self.conditions.len() {
            return Err(crate::calculator::index_out_of_range());
        }
        if self.item_type() == Types::Else {
            if ttype == Types::Else {
                return Err(exceptions::DuplicateElse());
            }
            if ttype == Types::ElseIf {
                return Err(exceptions::ElseIfAfterElse());
            }
        }
        match ttype {
            Types::If => self.add(true, line, source_line)?,
            Types::While => {
                self.id += 1;
                self.ensure(self.id)?;
                self.conditions[self.id] = Item {
                    value: true,
                    ttype,
                    line,
                    source_line,
                };
                self.count += 1;
            }
            Types::ElseIf => self.change(true, Types::If),
            Types::Else => {
                let v = !self.is_found;
                self.change(v, ttype)
            }
            Types::EndIf => self.remove(),
            Types::None => {}
        }
        Ok(())
    }

    pub fn check(&mut self, value: Complex) -> CResult<()> {
        let result = Condition::is_true(value)?;
        if result {
            self.is_found = true;
        }
        let t = self.item_type();
        self.change(result, t);
        self.is_unchecked = false;
        Ok(())
    }

    /// Deviation from CalcpadCE: a condition whose expression failed leaves the whole
    /// `#if` chain unresolved, so neither this branch nor any `#else if`/`#else` runs.
    pub fn check_failed(&mut self) {
        self.is_found = true;
        let t = self.item_type();
        self.change(false, t);
        self.is_unchecked = false;
    }

    pub fn is_true(value: Complex) -> CResult<bool> {
        if !value.is_real() {
            return Err(exceptions::ConditionComplex());
        }
        let d = value.re();
        if d.is_nan() || d.is_infinite() {
            return Err(exceptions::ConditionResultInvalid(&netfmt::double_to_string(d)));
        }
        Ok(d.abs() > 1e-12)
    }

    pub fn check_unchecked(&mut self) {
        self.is_unchecked = false;
    }

    pub fn to_html(&self) -> String {
        if self.keyword.is_empty() {
            return String::new();
        }
        format!("<span class=\"cond\">{}</span>", self.keyword)
    }
}

impl Types {
    fn ord(self) -> i32 {
        self as i32
    }
}

impl PartialOrd for Types {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.ord().cmp(&other.ord()))
    }
}

fn get_conditional_keyword(t: Types) -> &'static str {
    match t {
        Types::If => "#if ",
        Types::ElseIf => "#else if ",
        Types::Else => "#else",
        Types::EndIf => "#end if",
        Types::While => "#while ",
        Types::None => "",
    }
}
