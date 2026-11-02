//! Port of Parsers/MathParser/MathParser.SyntaxAnalyser.cs.
use super::is_assignment;
use super::tokens::{Token, TokenRef, TokenTypes};
use super::MathParser;
use crate::calculator::NEG_STR;
use crate::exceptions;
use crate::math_parser_exception::CResult;

#[derive(Clone)]
struct StackItem {
    token: TokenRef,
    count_of_brackets: i32,
    count_of_divisors: i32,
}

fn order_index(t: TokenTypes) -> usize {
    use TokenTypes::*;
    match t {
        None | Error => 0,
        Solver | Constant | Variable | Unit | Input => 1,
        Operator => 2,
        Function
        | Function2
        | Function3
        | MultiFunction
        | Interpolation
        | VectorFunction
        | VectorFunction2
        | VectorFunction3
        | VectorMultiFunction
        | MatrixFunction
        | MatrixFunction2
        | MatrixFunction3
        | MatrixFunction4
        | MatrixFunction5
        | MatrixIterativeFunction
        | MatrixOptionalFunction
        | MatrixMultiFunction
        | CustomFunction => 3,
        BracketLeft | SquareBracketLeft => 4,
        BracketRight | SquareBracketRight => 5,
        Divisor | RowDivisor => 6,
        Vector | Matrix | Array => 7,
        VectorIndex | MatrixIndex | ArrayIndex => 8,
    }
}

const INDEX_ORDER: usize = 8;
const FUNCTION_ORDER: usize = 3;

const CORRECT_ORDER: [[bool; 9]; 9] = [
    // None   Const  Oper   Func   Left   Right  Div    Vec    Index
    [true, true, true, true, true, true, true, true, false], // None
    [true, false, true, false, false, true, true, false, false], // Constant
    [true, true, false, true, true, false, false, true, false], // Operator
    [true, false, false, false, true, false, false, false, false], // Function
    [true, true, false, true, true, false, false, true, false], // Left Bracket
    [true, false, true, false, false, true, true, false, true], // Right Bracket
    [true, true, false, true, true, false, false, true, false], // Divisor
    [true, false, true, false, false, true, true, false, true], // Vector
    [false, true, false, true, true, false, false, true, false], // Index
];

fn check_order(pt: &Token, ct: &Token) -> CResult<()> {
    let mut ptt = pt.ttype;
    let mut ctt = ct.ttype;
    if pt.content == NEG_STR {
        ptt = TokenTypes::Operator;
    } else if pt.content == "!" {
        ptt = TokenTypes::Constant;
    } else if ct.content == "!" {
        ctt = TokenTypes::Operator;
    } else if ct.content == NEG_STR {
        ctt = TokenTypes::Function;
    }
    if !CORRECT_ORDER[order_index(ptt)][order_index(ctt)] {
        return Err(exceptions::InvalidSyntax2(&pt.content, &ct.content));
    }
    Ok(())
}

impl MathParser {
    /// `SyntaxAnalyser.Check`; returns `isFunctionDefinition`.
    pub(crate) fn syntax_check(&self, input: &[TokenRef], is_calculating: bool) -> CResult<bool> {
        let mut is_function_definition = false;
        if input.is_empty() {
            return Ok(false);
        }
        let mut count_of_brackets = 0i32;
        let mut count_of_operators = 0i32;
        let mut count_of_divisors = 0i32;
        let mut index_num = 0i32;
        let mut is_function_in_index = false;
        let mut multi_function_stack: Vec<StackItem> = Vec::new();
        let mut optional_function_stack: Vec<StackItem> = Vec::new();
        let mut vector_stack: Vec<StackItem> = Vec::new();
        let mut index_stack: Vec<StackItem> = Vec::new();
        let mut pt: TokenRef = Token::new("", TokenTypes::None).into_ref();
        let first_token = input[0].clone();
        for tr in input {
            let tt = tr.borrow().ttype;
            match tt {
                TokenTypes::Function2 | TokenTypes::VectorFunction2 | TokenTypes::MatrixFunction2 => {
                    count_of_divisors -= 1
                }
                TokenTypes::MatrixIterativeFunction => {
                    let index = tr.borrow().index;
                    if crate::calculator::matrix_calculator::is_last_parameter_optional(index) {
                        optional_function_stack.push(StackItem {
                            token: tr.clone(),
                            count_of_brackets,
                            count_of_divisors,
                        });
                    }
                    count_of_divisors -= 1;
                }
                TokenTypes::Function3 | TokenTypes::VectorFunction3 | TokenTypes::MatrixFunction3 => {
                    count_of_divisors -= 2
                }
                TokenTypes::MatrixFunction4 => count_of_divisors -= 3,
                TokenTypes::MatrixFunction5 => count_of_divisors -= 4,
                TokenTypes::MultiFunction
                | TokenTypes::Interpolation
                | TokenTypes::VectorMultiFunction
                | TokenTypes::MatrixMultiFunction => multi_function_stack.push(StackItem {
                    token: tr.clone(),
                    count_of_brackets,
                    count_of_divisors,
                }),
                TokenTypes::CustomFunction => {
                    let (index, is_ft, content, pc) = {
                        let t = tr.borrow();
                        (t.index, t.is_function_token(), t.content.clone(), t.parameter_count())
                    };
                    if index < 0 {
                        if is_function_definition && content == first_token.borrow().content {
                            return Err(exceptions::RecursionNotAllowed(&content));
                        }
                        if is_ft {
                            multi_function_stack.push(StackItem {
                                token: tr.clone(),
                                count_of_brackets,
                                count_of_divisors,
                            });
                        }
                    } else {
                        let n = if is_ft {
                            pc
                        } else {
                            self.functions
                                .borrow()
                                .get(index)
                                .map(|f| f.parameter_count() as i32)
                                .unwrap_or(0)
                        };
                        count_of_divisors += 1 - n;
                    }
                }
                TokenTypes::BracketLeft => {
                    if index_num != 0 {
                        index_stack.push(StackItem {
                            token: pt.clone(),
                            count_of_brackets,
                            count_of_divisors,
                        });
                        if order_index(pt.borrow().ttype) == INDEX_ORDER {
                            count_of_divisors += 1 - index_num;
                        } else {
                            is_function_in_index = true;
                        }
                    }
                    count_of_brackets += 1;
                }
                TokenTypes::BracketRight => {
                    count_of_brackets -= 1;
                    if count_of_brackets < 0 {
                        return Err(exceptions::MissingLeftBracket());
                    }
                    if multi_function_stack
                        .last()
                        .is_some_and(|m| count_of_brackets == m.count_of_brackets)
                    {
                        let mf = multi_function_stack.pop().unwrap();
                        let is_ft = mf.token.borrow().is_function_token();
                        if is_ft {
                            mf.token
                                .borrow_mut()
                                .set_parameter_count(count_of_divisors - mf.count_of_divisors + 1);
                        } else {
                            return Err(exceptions::InvalidFunction(&mf.token.borrow().content));
                        }
                        count_of_divisors = mf.count_of_divisors;
                    } else if index_stack
                        .last()
                        .is_some_and(|m| count_of_brackets == m.count_of_brackets)
                    {
                        let index = index_stack.pop().unwrap();
                        if is_function_in_index {
                            is_function_in_index = false;
                        } else {
                            let index_divisors_count = count_of_divisors - index.count_of_divisors;
                            let itt = index.token.borrow().ttype;
                            if itt == TokenTypes::ArrayIndex {
                                let n = index_divisors_count - 1;
                                if n == 1 {
                                    index.token.borrow_mut().ttype = TokenTypes::VectorIndex;
                                } else if n == 2 {
                                    index.token.borrow_mut().ttype = TokenTypes::MatrixIndex;
                                } else {
                                    return Err(exceptions::InvalidNumberOfIndexes());
                                }
                            } else if index_divisors_count != 0 {
                                return Err(exceptions::InvalidNumberOfIndexes());
                            }
                            count_of_divisors = index.count_of_divisors;
                        }
                    } else if optional_function_stack.last().is_some_and(|m| {
                        count_of_brackets == m.count_of_brackets && count_of_divisors == m.count_of_divisors - 1
                    }) {
                        count_of_divisors += 1;
                        let of = optional_function_stack.last().unwrap();
                        of.token.borrow_mut().ttype = TokenTypes::MatrixOptionalFunction;
                    }
                }
                TokenTypes::SquareBracketLeft => vector_stack.push(StackItem {
                    token: tr.clone(),
                    count_of_brackets,
                    count_of_divisors,
                }),
                TokenTypes::SquareBracketRight => {
                    let item = match vector_stack.pop() {
                        None => return Err(exceptions::MissingVectorOpeningBracket()),
                        Some(i) => i,
                    };
                    if count_of_brackets != item.count_of_brackets {
                        return Err(exceptions::BracketMismatch());
                    }
                    count_of_divisors = item.count_of_divisors;
                }
                TokenTypes::Divisor => count_of_divisors += 1,
                TokenTypes::Operator => {
                    if index_stack.is_empty() {
                        count_of_operators += 1;
                    }
                    if is_assignment(&tr.borrow().content) {
                        let ftt = first_token.borrow().ttype;
                        if ftt == TokenTypes::CustomFunction {
                            count_of_divisors = 0;
                            is_function_definition = true;
                        } else if is_calculating {
                            let ptt = pt.borrow().ttype;
                            if ptt != TokenTypes::Variable
                                && ptt != TokenTypes::Unit
                                && ftt != TokenTypes::Vector
                                && ftt != TokenTypes::Matrix
                                && ftt != TokenTypes::Array
                            {
                                if ftt == TokenTypes::Function {
                                    return Err(exceptions::BuiltInFunctionRedefined(&first_token.borrow().content));
                                }
                                return Err(exceptions::AssignmentPreceded());
                            } else if count_of_operators != 1 {
                                return Err(exceptions::AssignmentNotFirst());
                            }
                        }
                    }
                }
                TokenTypes::VectorIndex => index_num = 1,
                TokenTypes::MatrixIndex => index_num = 2,
                TokenTypes::ArrayIndex => index_num = -1,
                _ => {}
            }
            {
                let p = pt.borrow();
                let t = tr.borrow();
                check_order(&p, &t)?;
            }
            let fix = {
                let p = pt.borrow();
                p.ttype == TokenTypes::ArrayIndex && tt != TokenTypes::BracketLeft
            };
            if fix {
                pt.borrow_mut().ttype = TokenTypes::VectorIndex;
            }
            pt = tr.clone();
            if index_num != 0 {
                let order = order_index(tr.borrow().ttype);
                if order != INDEX_ORDER && order != FUNCTION_ORDER {
                    index_num = 0;
                }
            }
        }
        let mut pt: Option<TokenRef> = None;
        for tr in input {
            if let Some(p) = &pt {
                let pa = p.borrow().ttype == TokenTypes::Array;
                if pa {
                    let tt = tr.borrow().ttype;
                    if tt == TokenTypes::VectorIndex {
                        p.borrow_mut().ttype = TokenTypes::Vector;
                    } else if tt == TokenTypes::MatrixIndex {
                        p.borrow_mut().ttype = TokenTypes::Matrix;
                    }
                }
            }
            pt = Some(tr.clone());
        }
        let last = pt.unwrap();
        let (ptt, pcontent) = {
            let l = last.borrow();
            (l.ttype, l.content.clone())
        };
        let ok_last = {
            use TokenTypes::*;
            matches!(
                ptt,
                None | Constant
                    | Variable
                    | Unit
                    | Input
                    | Vector
                    | Matrix
                    | Array
                    | BracketRight
                    | SquareBracketRight
                    | Solver
                    | Error
            )
        };
        if !ok_last && pcontent != "!" {
            return Err(exceptions::IncompleteExpression());
        }
        let (ftt, fidx, fcontent) = {
            let f = first_token.borrow();
            (f.ttype, f.index, f.content.clone())
        };
        if ftt == TokenTypes::CustomFunction && fidx < 0 && !is_function_definition && is_calculating {
            return Err(exceptions::InvalidFunction(&fcontent));
        }
        if count_of_brackets > 0 {
            return Err(exceptions::MissingRightBracket());
        }
        if !vector_stack.is_empty() {
            return Err(exceptions::MissingVectorClosingBracket());
        }
        if count_of_divisors > 0 {
            return Err(exceptions::UnexpectedDelimiter());
        }
        if count_of_divisors < 0 {
            return Err(exceptions::InvalidNumberOfArguments());
        }
        Ok(is_function_definition)
    }
}
