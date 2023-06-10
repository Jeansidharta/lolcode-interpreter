use std::fs::read_to_string;
use std::path::PathBuf;

use lolcode_ast::lexer::{NumberToken, TokenType, TokenValue};
use lolcode_ast::parser::expression::{ASTExpression, Identifier};
use lolcode_ast::parser::statements::assignment::VariableAssignment;
use lolcode_ast::parser::statements::bukkit_set_slot::BukkitSetSlot;
use lolcode_ast::parser::statements::i_has_a::{IHasA, IHasAInitialValue};
use lolcode_ast::parser::statements::i_is::IIz;
use lolcode_ast::parser::statements::im_in_yr::{
    ImInYr, LoopCondition, LoopIterationOperation, LoopOperation,
};
use lolcode_ast::parser::statements::o_rly::ORly;
use lolcode_ast::parser::statements::visible::Visible;
use lolcode_ast::parser::statements::ASTNode;
use lolcode_ast::parser::ASTBlock;

use lolcode_ast::parser::statements::wtf::Wtf;
use types::{RuntimeError, RuntimeResult, Scope, Value};
use variable_access::{
    get_variable_access_from_scope, mutate_variable_access, write_identifier_to_scope,
};

mod types;
mod variable_access;

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Value::Noob => "NOOB".to_string(),
            Value::Yarn(yarn) => yarn.clone(),
            Value::Troof(troof) => troof.to_string(),
            Value::Numbr(numbr) => numbr.to_string(),
            Value::Numbar(numbr) => numbr.to_string(),
            Value::Bukkit(bukkit) => bukkit.to_string(),
        };
        write!(f, "{}", string)
    }
}

pub fn execute_statement(scope: &mut Scope, node: &ASTNode) -> RuntimeResult<Option<Value>> {
    Ok(match node {
        ASTNode::HAI(_) => None,
        ASTNode::IHasA(IHasA {
            identifier,
            initial_value,
        }) => {
            let initial_value = match initial_value {
                Some(IHasAInitialValue::Expression(e)) => parse_expression(scope, e)?,
                Some(IHasAInitialValue::Type(t)) => t.into(),
                None => Value::Noob,
            };
            write_identifier_to_scope(scope, identifier, initial_value)?;
            None
        }
        ASTNode::ImInYr(ImInYr {
            on_iteration,
            condition,
            code_block,
            ..
        }) => {
            let mut while_scope = scope.child();
            while match condition {
                None => true,
                Some(LoopCondition::TIL(expression)) => {
                    !parse_expression(&while_scope, expression)?.to_boolean()
                }
                Some(LoopCondition::WILE(expression)) => {
                    parse_expression(&while_scope, expression)?.to_boolean()
                }
            } {
                execute_block(&mut while_scope, &code_block)?;
                if let Some(LoopIterationOperation { operation, operand }) = on_iteration {
                    match operation {
                        LoopOperation::UPPIN(_) => mutate_variable_access(
                            &while_scope,
                            operand,
                            Box::new(|operand| match operand {
                                Value::Numbr(num) => Ok((num + 1).into()),
                                Value::Numbar(num) => Ok((num + 1f32).into()),
                                _ => Err(RuntimeError::IdentifierNotFound),
                            }),
                        )?,
                        LoopOperation::NERFIN(_) => mutate_variable_access(
                            &while_scope,
                            operand,
                            Box::new(|operand| match operand {
                                Value::Numbr(num) => Ok((num - 1).into()),
                                Value::Numbar(num) => Ok((num - 1f32).into()),
                                _ => Err(RuntimeError::IdentifierNotFound),
                            }),
                        )?,
                    };
                };
            }
            None
        }
        ASTNode::BukkitSetSlot(BukkitSetSlot {
            bukkit,
            slot_name,
            value,
        }) => {
            let mut bukkit = get_variable_access_from_scope(scope, bukkit)?;
            let expression_value = parse_expression(scope, value)?;
            match *bukkit {
                Value::Bukkit(ref mut bukkit) => bukkit
                    .0
                    .insert(slot_name.to_string_slice().into(), expression_value),
                _ => return Err(RuntimeError::InvalidType),
            };
            None
        }
        ASTNode::VariableAssignment(VariableAssignment {
            variable_access,
            expression,
        }) => {
            let value = parse_expression(scope, expression)?;
            mutate_variable_access(scope, variable_access, Box::new(|_| Ok(value)))?;
            None
        }
        ASTNode::Visible(Visible(expressions, has_exclamation)) => {
            for (index, expression) in expressions.into_iter().enumerate() {
                let value = parse_expression(scope, expression)?;
                if index == 0 {
                    print!("{}", value);
                } else {
                    print!(" {}", value);
                }
            }
            if has_exclamation.is_none() {
                println!("");
            }
            None
        }
        ASTNode::FoundYr(expr) => Some(parse_expression(scope, expr)?),
        ASTNode::Wtf(Wtf { omg, omg_wtf }) => {
            let mut found = false;
            for (expression, block) in omg.into_iter() {
                let value = parse_expression(scope, expression)?;
                if value == *scope.it.borrow() {
                    execute_block(&mut scope.child(), block)?;
                    found = true;
                    break;
                }
            }
            if let Some(block) = omg_wtf {
                if !found {
                    execute_block(&mut scope.child(), block)?;
                }
            }
            None
        }
        ASTNode::ORly(ORly {
            if_true,
            if_false,
            mebbes,
        }) => 'block: {
            if let Some(block) = if_true {
                if scope.it.borrow().to_boolean() {
                    execute_block(&mut scope.child(), block)?;
                    break 'block None;
                }
            }
            for (expression, block) in mebbes.into_iter() {
                let expression_value = parse_expression(scope, expression)?;
                if expression_value.to_boolean() {
                    execute_block(&mut scope.child(), block)?;
                    break 'block None;
                }
            }
            if let Some(block) = if_false {
                execute_block(scope, block)?;
            }
            None
        }
        ASTNode::IIz(IIz {
            name: _,
            arguments: _,
        }) => todo!(),
        ASTNode::HowIzI(_) => todo!(),
        ASTNode::Gtfo(_) => Some(Value::Noob),
        ASTNode::Gimmeh(variable) => {
            let mut input = String::new();
            std::io::stdin()
                .read_line(&mut input)
                .expect("Could not read user input");
            mutate_variable_access(scope, variable, Box::new(|_| Ok(input.into())))?;
            None
        }
        ASTNode::Expression(expression) => {
            let value = parse_expression(scope, expression)?;
            scope.it = value.into();
            None
        }
        ASTNode::ASTError(_) => todo!(),
        ASTNode::KTHXBYE(_) => None,
    })
}

pub fn execute_block(scope: &mut Scope, block: &ASTBlock) -> RuntimeResult<Value> {
    for statement in block.0.iter() {
        match execute_statement(scope, statement)? {
            Some(value) => return Ok(value),
            None => {}
        };
    }

    Ok(Value::Noob)
}

pub fn execute_file(file: PathBuf) -> Result<(), String> {
    let file = read_to_string(file).map_err(|_| "Failed to read source code".to_string())?;
    let ast = lolcode_ast::tokenize_and_parse(file).map_err(|err| format!("{:?}", err))?;
    let mut scope = Scope::default();

    for node in ast.into_iter() {
        execute_statement(&mut scope, &node).map_err(|_err| "runtime error".to_string())?;
    }

    let _root_scope = Scope::default();

    Ok(())
}

fn parse_expression(scope: &Scope, expression: &ASTExpression) -> RuntimeResult<Value> {
    Ok(match expression {
        ASTExpression::LiteralValue(val) => match &val.token_type {
            TokenType::Value(val) => match val {
                TokenValue::NOOB => Value::Noob,
                TokenValue::Number(num) => match num {
                    NumberToken::Int(int) => Value::Numbr(*int),
                    NumberToken::Float(float) => Value::Numbar(*float),
                },
                TokenValue::String(string) => Value::Yarn(string.clone()),
                TokenValue::Boolean(bool) => Value::Troof(*bool),
            },
            _ => unreachable!(),
        },
        ASTExpression::VariableAccess(variable_access) => {
            get_variable_access_from_scope(scope, variable_access)?.clone()
        }
        ASTExpression::BothOf(left, right) => (parse_expression(scope, left)?.to_boolean()
            && parse_expression(scope, right)?.to_boolean())
        .into(),
        ASTExpression::EitherOf(left, right) => (parse_expression(scope, left)?.to_boolean()
            || parse_expression(scope, right)?.to_boolean())
        .into(),
        ASTExpression::WonOf(left, right) => (parse_expression(scope, left)?.to_boolean()
            != parse_expression(scope, right)?.to_boolean())
        .into(),
        ASTExpression::Not(expression) => parse_expression(scope, expression)?.not().into(),
        ASTExpression::AllOf(values) => {
            for elem in values.into_iter() {
                if !parse_expression(scope, elem)?.to_boolean() {
                    return Ok(false.into());
                }
            }
            return Ok(true.into());
        }
        ASTExpression::AnyOf(values) => {
            for elem in values.into_iter() {
                if parse_expression(scope, elem)?.to_boolean() {
                    return Ok(true.into());
                }
            }
            return Ok(false.into());
        }
        ASTExpression::SumOf(left, right) => {
            let left = parse_expression(scope, left)?;
            let right = parse_expression(scope, right)?;
            match (left, right) {
                (Value::Numbar(l), Value::Numbar(r)) => Value::Numbar(l + r),
                (Value::Numbr(l), Value::Numbar(r)) => Value::Numbar(l as f32 + r),
                (Value::Numbar(l), Value::Numbr(r)) => Value::Numbar(l + r as f32),
                (Value::Numbr(l), Value::Numbr(r)) => Value::Numbr(l + r),
                _ => return Err(RuntimeError::InvalidType),
            }
        }
        ASTExpression::DiffOf(left, right) => {
            let left = parse_expression(scope, left)?;
            let right = parse_expression(scope, right)?;
            match (left, right) {
                (Value::Numbar(l), Value::Numbar(r)) => Value::Numbar(l - r),
                (Value::Numbr(l), Value::Numbar(r)) => Value::Numbar(l as f32 - r),
                (Value::Numbar(l), Value::Numbr(r)) => Value::Numbar(l - r as f32),
                (Value::Numbr(l), Value::Numbr(r)) => Value::Numbr(l - r),
                _ => return Err(RuntimeError::InvalidType),
            }
        }
        ASTExpression::ProduktOf(left, right) => {
            let left = parse_expression(scope, left)?;
            let right = parse_expression(scope, right)?;
            match (left, right) {
                (Value::Numbar(l), Value::Numbar(r)) => Value::Numbar(l * r),
                (Value::Numbr(l), Value::Numbar(r)) => Value::Numbar(l as f32 * r),
                (Value::Numbar(l), Value::Numbr(r)) => Value::Numbar(l * r as f32),
                (Value::Numbr(l), Value::Numbr(r)) => Value::Numbr(l * r),
                _ => return Err(RuntimeError::InvalidType),
            }
        }
        ASTExpression::QuoshuntOf(left, right) => {
            let left = parse_expression(scope, left)?;
            let right = parse_expression(scope, right)?;
            match (left, right) {
                (Value::Numbar(l), Value::Numbar(r)) => Value::Numbar(l / r),
                (Value::Numbr(l), Value::Numbar(r)) => Value::Numbar(l as f32 / r),
                (Value::Numbar(l), Value::Numbr(r)) => Value::Numbar(l / r as f32),
                (Value::Numbr(l), Value::Numbr(r)) => Value::Numbr(l / r),
                _ => return Err(RuntimeError::InvalidType),
            }
        }
        ASTExpression::ModOf(left, right) => {
            let left = parse_expression(scope, left)?;
            let right = parse_expression(scope, right)?;
            match (left, right) {
                (Value::Numbar(l), Value::Numbar(r)) => Value::Numbar(l % r),
                (Value::Numbr(l), Value::Numbar(r)) => Value::Numbar(l as f32 % r),
                (Value::Numbar(l), Value::Numbr(r)) => Value::Numbar(l % r as f32),
                (Value::Numbr(l), Value::Numbr(r)) => Value::Numbr(l % r),
                _ => return Err(RuntimeError::InvalidType),
            }
        }
        ASTExpression::BiggrOf(left, right) => {
            let left = parse_expression(scope, left)?;
            let right = parse_expression(scope, right)?;
            match (left, right) {
                (Value::Numbar(l), Value::Numbar(r)) => Value::Troof(l > r),
                (Value::Numbr(l), Value::Numbar(r)) => Value::Troof(l as f32 > r),
                (Value::Numbar(l), Value::Numbr(r)) => Value::Troof(l > r as f32),
                (Value::Numbr(l), Value::Numbr(r)) => Value::Troof(l > r),
                _ => return Err(RuntimeError::InvalidType),
            }
        }
        ASTExpression::SmallrOf(left, right) => {
            let left = parse_expression(scope, left)?;
            let right = parse_expression(scope, right)?;
            match (left, right) {
                (Value::Numbar(l), Value::Numbar(r)) => Value::Troof(l < r),
                (Value::Numbr(l), Value::Numbar(r)) => Value::Troof((l as f32) < r),
                (Value::Numbar(l), Value::Numbr(r)) => Value::Troof(l < r as f32),
                (Value::Numbr(l), Value::Numbr(r)) => Value::Troof(l < r),
                _ => return Err(RuntimeError::InvalidType),
            }
        }
        ASTExpression::BothSaem(left, right) => {
            (parse_expression(scope, left)? == parse_expression(scope, right)?).into()
        }
        ASTExpression::Diffrint(left, right) => {
            (parse_expression(scope, left)? == parse_expression(scope, right)?).into()
        }
        ASTExpression::Smoosh(values) => {
            let mut result = String::new();
            for value in values.into_iter() {
                let value = parse_expression(scope, value)?;
                match value {
                    Value::Yarn(string) => result.push_str(&string),
                    _ => return Err(RuntimeError::InvalidType),
                }
            }
            Value::Yarn(result)
        }
        ASTExpression::Maek(_, _) => todo!(),
    })
}
