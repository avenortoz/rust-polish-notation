use core::cmp::PartialEq;
use log::debug;
use std::error::Error;
use std::str;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum Operation {
    Add,
    Substract,
    Divide,
    Multiply,
    Power,
}

#[derive(Debug, PartialEq)]
pub enum Symbol {
    Number(f64),
    Op(Operation),
    OpenBracket,
    CloseBracket,
}

pub struct Config {
    expression: String,
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        args.next();
        let expression = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get an expression field"),
        };
        Ok(Config { expression })
    }
}

pub fn run(config: Config) -> Result<f64, Box<dyn Error>> {
    let tokens = build_polish_notation(&config.expression)?;
    Ok(evaluate(tokens)?)
}

fn build_polish_notation(expression: &str) -> Result<Vec<Symbol>, &'static str> {
    let mut res: Vec<Symbol> = Vec::new();
    let mut stack: Vec<Symbol> = Vec::new();

    let chars: Vec<_> = expression.bytes().collect();
    let mut i = 0;
    while i < chars.len() {
        match &chars[i] {
            op @ (b'+' | b'-' | b'*' | b'/' | b'^') => {
                debug!("Got operation: {}", *op as char);
                let new_op = match *op {
                    b'+' => Symbol::Op(Operation::Add),
                    b'-' => Symbol::Op(Operation::Substract),
                    b'*' => Symbol::Op(Operation::Multiply),
                    b'/' => Symbol::Op(Operation::Divide),
                    b'^' => Symbol::Op(Operation::Power),
                    _ => todo!(),
                };
                while stack.len() > 0 {
                    let token = stack.pop().unwrap();
                    match token {
                        op @ Symbol::Op(Operation::Power) => {
                            res.push(op);
                        }
                        op @ (Symbol::Op(Operation::Divide) | Symbol::Op(Operation::Multiply)) => {
                            match new_op {
                                Symbol::Op(Operation::Add)
                                | Symbol::Op(Operation::Substract)
                                | Symbol::Op(Operation::Multiply)
                                | Symbol::Op(Operation::Divide) => {
                                    res.push(op);
                                }
                                _ => {
                                    stack.push(op);
                                    break;
                                }
                            };
                        }
                        _ => {
                            stack.push(token);
                            break;
                        }
                    }
                }
                stack.push(new_op);
            }
            b'1' | b'2' | b'3' | b'4' | b'5' | b'6' | b'7' | b'8' | b'9' | b'0' => {
                let number_start = i;
                let mut number_end = number_start + 1;
                let mut is_dot_visited = false;
                while number_end < chars.len() {
                    match chars[number_end] {
                        b'1' | b'2' | b'3' | b'4' | b'5' | b'6' | b'7' | b'8' | b'9' | b'0' => {
                            number_end += 1;
                        }
                        b'.' => {
                            if is_dot_visited {
                                return Err("dot two time");
                            } else {
                                is_dot_visited = true;
                                number_end += 1;
                            }
                        }
                        b'+' | b'-' | b'*' | b'/' | b'^' | b' ' | b')' | b'(' => {
                            break;
                        }
                        _ => {
                            return Err("Non valid float");
                        }
                    }
                }

                // number number_start..=number_end
                let num_str = str::from_utf8(&chars[number_start..number_end]).unwrap();
                debug!("{}", num_str);
                let num = f64::from_str(num_str).unwrap();
                res.push(Symbol::Number(num));
                i = number_end - 1;
            }
            b'(' => {
                stack.push(Symbol::OpenBracket);
                debug!("OpenBracket");
            }
            b')' => {
                debug!("CloseBracket");
                loop {
                    let token = stack.pop().unwrap();
                    if let Symbol::OpenBracket = token {
                        break;
                    } else {
                        res.push(token);
                    }
                }
            }
            b' ' => debug!("Whitespace"),
            _ => {
                debug!("Something else");
                return Err("Not a valid symbol");
            }
        }
        i += 1;
    }
    while stack.len() > 0 {
        let token = stack.pop().unwrap();
        match token {
            Symbol::CloseBracket | Symbol::OpenBracket => return Err("Wrong input expresion"),
            _ => {
                res.push(token);
            }
        }
    }
    return Ok(res);
}

fn evaluate(rpn: Vec<Symbol>) -> Result<f64, &'static str> {
    let mut stack: Vec<Symbol> = Vec::new();
    for token in rpn {
        if let Symbol::Number(num) = token {
            stack.push(Symbol::Number(num));
        }
        if let Symbol::Op(operation) = token {
            let a = match stack.pop() {
                Some(Symbol::Number(val)) => val,
                _ => return Err("Not valid rpn expression"),
            };
            let b = match stack.pop() {
                Some(Symbol::Number(val)) => val,
                _ => return Err("Not valid rpn expression"),
            };
            let comb = match operation {
                Operation::Add => Symbol::Number(a + b),
                Operation::Substract => Symbol::Number(b - a),
                Operation::Divide => {
                    if a.abs() < 1e-10 {
                        return Err("Zero devision");
                    }
                    Symbol::Number(b / a)
                }
                Operation::Multiply => Symbol::Number(a * b),
                Operation::Power => Symbol::Number(b.powf(a)),
            };
            stack.push(comb)
        }
    }
    match stack[0] {
        Symbol::Number(val) => Ok(val),
        _ => Err("Evaluation ended with something but number"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero() {
        let stack = vec![
            Symbol::Number(5f64),
            Symbol::Number(10f64),
            Symbol::Op(Operation::Add),
        ];
        assert_eq!(15f64, evaluate(stack).unwrap());
    }

    #[test]
    fn zero_rpn() {
        let expression = "5+10";
        let stack = vec![
            Symbol::Number(5f64),
            Symbol::Number(10f64),
            Symbol::Op(Operation::Add),
        ];

        let res = build_polish_notation(&expression).unwrap();
        assert_eq!(stack, res);
    }

    #[test]
    fn wikipidia_test() {
        // reverse postfix nototation:
        // 12 2 3 4 * 10 5 / + * +

        let stack = vec![
            Symbol::Number(12f64),
            Symbol::Number(2f64),
            Symbol::Number(3f64),
            Symbol::Number(4f64),
            Symbol::Op(Operation::Multiply),
            Symbol::Number(10f64),
            Symbol::Number(5f64),
            Symbol::Op(Operation::Divide),
            Symbol::Op(Operation::Add),
            Symbol::Op(Operation::Multiply),
            Symbol::Op(Operation::Add),
        ];

        assert_eq!(40f64, evaluate(stack).unwrap());
    }

    #[test]
    fn wikipidia_test_build_rpn() {
        // Input
        // 12 + 2 * ( ( 3 * 4 ) + ( 10 / 5 ) )
        let expression = "12 + 2 * ( ( 3 * 4 ) + ( 10 / 5 ) )".to_string();

        // Res
        // 12 2 3 4 * 10 5 / + * +
        //let mut stack = Vec::new();
        let mut stack = vec![];
        stack.push(Symbol::Number(12f64));
        stack.push(Symbol::Number(2f64));
        stack.push(Symbol::Number(3f64));
        stack.push(Symbol::Number(4f64));
        stack.push(Symbol::Op(Operation::Multiply));
        stack.push(Symbol::Number(10f64));
        stack.push(Symbol::Number(5f64));
        stack.push(Symbol::Op(Operation::Divide));
        stack.push(Symbol::Op(Operation::Add));
        stack.push(Symbol::Op(Operation::Multiply));
        stack.push(Symbol::Op(Operation::Add));

        let res = build_polish_notation(&expression).unwrap();
        assert_eq!(stack, res);
    }
}
