use lolcode_ast::parser::expression::VariableAccess;

use crate::{Identifier, RuntimeError, RuntimeResult, Scope, Value};
use std::cell::{Ref, RefMut};

pub fn get_scope_that_has_value<'a>(
    mut scope: &'a Scope,
    ident_name: &str,
) -> Option<&'a Scope<'a>> {
    loop {
        if scope.variables.borrow().contains_key(ident_name) {
            break Some(scope);
        }

        scope = match scope.parent_scope {
            None => break None,
            Some(scope) => scope,
        };
    }
}

pub fn get_scope_that_has_value_srs<'a>(
    scope: &'a Scope,
    ident_name: &str,
    is_srs: bool,
) -> Option<&'a Scope<'a>> {
    let scope = get_scope_that_has_value(scope, ident_name)?;

    if !is_srs {
        return Some(scope);
    };

    let value = Ref::map(scope.variables.borrow(), |v| v.get(ident_name).unwrap());

    match *value {
        Value::Yarn(ref yarn) => get_scope_that_has_value(scope, &yarn),
        _ => return None,
    }
}

pub fn get_identifier_from_scope<'a>(
    scope: &'a Scope,
    identifier: &Identifier,
) -> RuntimeResult<RefMut<'a, Value>> {
    let ident_name = identifier.to_string_slice();
    let scope = match get_scope_that_has_value_srs(scope, ident_name, identifier.is_srs) {
        None => return Err(RuntimeError::IdentifierNotFound),
        Some(s) => s,
    };

    Ok(RefMut::map(scope.variables.borrow_mut(), |v| {
        v.get_mut(ident_name).unwrap()
    }))
}

pub fn write_identifier_to_scope<'a>(
    scope: &'a mut Scope,
    ident_name: &Identifier,
    initial_value: Value,
) -> RuntimeResult<()> {
    scope
        .variables
        .borrow_mut()
        .insert(ident_name.to_string_slice().to_string(), initial_value);

    Ok(())
}

pub fn mutate_variable_access(
    scope: &Scope,
    variable_access: &VariableAccess,
    mutator: Box<dyn FnOnce(&Value) -> RuntimeResult<Value>>,
) -> RuntimeResult<()> {
    let VariableAccess {
        name: identifier,
        accesses,
    } = variable_access;

    let mut value = &mut *get_identifier_from_scope(scope, identifier)?;

    for access in accesses.into_iter() {
        match value {
            Value::Bukkit(bukkit) => {
                value = bukkit
                    .0
                    .get_mut(access.to_string_slice())
                    .ok_or(RuntimeError::InvalidType)?;
            }
            _ => return Err(RuntimeError::InvalidType),
        };
    }

    let new_value = mutator(value)?;
    *value = new_value;

    Ok(())
}

pub fn get_variable_access_from_scope<'a>(
    scope: &'a Scope,
    variable_access: &VariableAccess,
) -> RuntimeResult<RefMut<'a, Value>> {
    let VariableAccess {
        name: identifier,
        accesses,
    } = variable_access;

    let mut value = get_identifier_from_scope(scope, identifier)?;

    for access in accesses.into_iter() {
        match *value {
            Value::Bukkit(ref bukkit) => {
                if bukkit.0.contains_key(access.to_string_slice()) {
                    return Err(RuntimeError::InvalidType);
                };
                value = RefMut::map(value, |bukit| match bukit {
                    Value::Bukkit(bukkit) => bukkit.0.get_mut(access.to_string_slice()).unwrap(),
                    _ => unreachable!(),
                });
            }
            _ => return Err(RuntimeError::InvalidType),
        };
    }
    Ok(value)
}
