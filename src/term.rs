
use std::time::{Instant, Duration};


#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Sort {
    Type,
    Kind
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Bind {
    Term,
    Type
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Term {
    Constant(Sort),
    Variable {
        index: i32
    },
    Application {
        function: Box<Term>,
        argument: Box<Term>
    },
    Binder {
        bind: Bind,
        type_annotation: Box<Term>,
        body: Box<Term>
    }
}

use Term::*;

fn shift(term:Term, cutoff:i32, amount:i32) -> Term {
    match term {
        Constant(sort) => Constant(sort),
        Variable { index } => {
            let shifted = if index < cutoff { index } else { index + amount };
            Variable { index:shifted }
        },
        Application { function, argument } => {
            let shifted_function = Box::new(shift(*function, cutoff, amount));
            let shifted_argument = Box::new(shift(*argument, cutoff, amount));
            Application { function:shifted_function, argument:shifted_argument }
        }
        Binder { bind, type_annotation, body } => {
            let shifted_body = Box::new(shift(*body, cutoff+1, amount));
            Binder { bind, type_annotation, body:shifted_body }
        }
    }
}

fn substitute(term:Term, value:Term, var: i32) -> Term {
    match term {
        Constant(sort) => Constant(sort),
        Variable { index } => {
            if index == var {
                term
            } else {
                Variable { index }
            }
        },
        Application { function, argument } => {
            // TODO: cloning the value here is potentially expensive
            let substituted_function = Box::new(substitute(*function, value.clone(), var));
            let substituted_argument = Box::new(substitute(*argument, value, var));
            Application { function:substituted_function, argument:substituted_argument }
        },
        Binder { bind, body, type_annotation } => {
            let substituted_body = Box::new(substitute(shift(*body, 0, 1), value, var+1));
            Binder { bind, body:substituted_body, type_annotation }
        },
    }
}

fn reduction_step(body:Term, argument:Term) -> Term {
    let shifted_argument = shift(argument, 1, 0);
    let result = substitute(body, shifted_argument, 0);
    shift(result, 0, -1)
}

// If the returned boolean is true then the term is in normal form
// otherwise the term is not in normal form
fn normalize_step(term:Term) -> (Term, bool) {
    match term {
        Constant(sort) => (Constant(sort), true),
        Variable { index } => (Variable { index }, true),
        Application { function, argument } => {
            let (function, is_function_normal) = normalize_step(*function);
            let (argument, is_argument_normal) = normalize_step(*argument);
            if let Binder { body, .. } = function {
                let reduced_term = reduction_step(*body, argument);
                (reduced_term, false)
            } else {
                let is_normal = is_function_normal && is_argument_normal;
                let function = Box::new(function);
                let argument = Box::new(argument);
                (Application { function, argument }, is_normal)
            }
        },
        Binder { bind, type_annotation, body } => {
            let (body, is_body_normal) = normalize_step(*body);
            let body = Box::new(body);
            (Binder { bind, type_annotation, body }, is_body_normal)
        }
    }
}

pub fn normalize(term:Term, timeout:Option<Duration>) -> Term {
    let timer = Instant::now();
    let mut result = term;
    let mut finished = false;

    while !finished {
        let (r, b) = normalize_step(result);
        result = r;
        finished = b || timeout.map_or(false, |d| timer.elapsed() > d);
    }

    result
}

pub fn infer(context:&mut Vec<Term>, term:Term) -> Result<Term, String> {
    match term {
        Constant(Sort::Type) => Ok(Constant(Sort::Kind)),
        Constant(Sort::Kind) => Err("Type Error: Kind does not have a type".to_owned()),
        Variable { index } =>
            context.get(context.len() - (index as usize) - 1)
                .map(|x| x.clone())
                .ok_or("Type Error: No type for variable".to_owned()),
        Application { function, argument } => {
            let fun_type = infer(context, *function)?;
            println!("function type: {:?}", fun_type);
            let arg_type = infer(context, *argument)?;
            match fun_type {
                Binder { bind, type_annotation, body } => {
                    let annotation = normalize(*type_annotation, None);
                    let arg_type = normalize(arg_type, None);
                    if annotation == arg_type && bind == Bind::Type {
                        Ok(substitute(*body, arg_type, 0))
                    } else {
                        Err("Type Error: Function type does not match argument type".to_owned())
                    }
                },
                _ => Err("Type Error: Function in application must be function typed".to_owned())
            }
        },
        Binder { bind:Bind::Term, type_annotation, body } => {
            let annotation = *type_annotation.clone();
            let arg_type = type_annotation.clone();
            infer(context, *type_annotation)?;
            context.push(annotation);
            let body_type = Box::new(infer(context, *body)?);
            context.pop();
            Ok(Binder { bind:Bind::Type, type_annotation:arg_type, body:body_type })
        },
        Binder { bind:Bind::Type, type_annotation, body } => {
            let annotation = *type_annotation.clone();
            infer(context, *type_annotation)?;
            context.push(annotation);
            let body_type = infer(context, *body)?;
            context.pop();
            Ok(body_type)
        }
    }
}
