use std::collections::{hash_map, HashMap};

use jsonata_errors::{Error, Result};

use crate::value;

use super::ast::*;
use super::frame::Frame;
use super::functions::*;
use super::value::{ArrayFlags, Value, ValueKind};

pub struct Evaluator {
    chain_ast: Ast,
}

impl Evaluator {
    pub fn new(chain_ast: Ast) -> Self {
        Evaluator { chain_ast }
    }

    fn fn_context<'a>(
        &'a self,
        name: &'a str,
        char_index: usize,
        input: Value,
        frame: &'a Frame,
    ) -> FunctionContext<'a> {
        FunctionContext {
            name,
            char_index,
            evaluator: self,
            input,
            frame: frame.clone(),
        }
    }

    pub fn evaluate(&self, node: &Ast, input: Value, frame: &Frame) -> Result<Value> {
        let mut result = match node.kind {
            AstKind::Null => Value::null(),
            AstKind::Bool(b) => Value::bool(b),
            AstKind::String(ref s) => Value::string(String::from(s)),
            AstKind::Number(n) => Value::number(n),
            AstKind::Block(ref exprs) => self.evaluate_block(exprs, input, frame)?,
            AstKind::Unary(ref op) => self.evaluate_unary_op(node, op, input, frame)?,
            AstKind::Binary(ref op, ref lhs, ref rhs) => {
                self.evaluate_binary_op(node, op, lhs, rhs, input, frame)?
            }
            AstKind::Var(ref name) => self.evaluate_var(name, input, frame)?,
            AstKind::Ternary {
                ref cond,
                ref truthy,
                ref falsy,
            } => self.evaluate_ternary(cond, truthy, falsy.as_deref(), input, frame)?,
            AstKind::Path(ref steps) => self.evaluate_path(node, steps, input, frame)?,
            AstKind::Name(ref name) => fn_lookup_internal(
                &self.fn_context("lookup", node.char_index, input, frame),
                input,
                name,
            ),
            AstKind::Lambda { .. } => Value::lambda(node, input, frame.clone()),
            AstKind::Function {
                ref proc,
                ref args,
                is_partial,
                ..
            } => self.evaluate_function(input, proc, args, is_partial, frame, None)?,

            _ => unimplemented!("TODO: node kind not yet supported: {:#?}", node.kind),
        };

        if let Some(filters) = &node.predicates {
            for filter in filters {
                result = self.evaluate_filter(filter, result, frame)?;
            }
        }

        Ok(if result.has_flags(ArrayFlags::SEQUENCE) {
            if node.keep_array {
                result.add_flags(ArrayFlags::SINGLETON);
            }
            if result.is_empty() {
                value::UNDEFINED
            } else if result.len() == 1 {
                if result.has_flags(ArrayFlags::SINGLETON) {
                    result
                } else {
                    result.get_member(0)
                }
            } else {
                result
            }
        } else {
            result
        })
    }

    fn evaluate_block(&self, exprs: &[Ast], input: Value, frame: &Frame) -> Result<Value> {
        let frame = Frame::new_with_parent(frame);
        if exprs.is_empty() {
            return Ok(value::UNDEFINED);
        }

        let mut result = value::UNDEFINED;
        for expr in exprs {
            result = self.evaluate(expr, input, &frame)?;
        }

        Ok(result)
    }

    fn evaluate_var(&self, name: &str, input: Value, frame: &Frame) -> Result<Value> {
        Ok(if name.is_empty() {
            if input.has_flags(ArrayFlags::WRAPPED) {
                input.get_member(0)
            } else {
                input
            }
        } else if let Some(value) = frame.lookup(name) {
            value
        } else {
            value::UNDEFINED
        })
    }

    fn evaluate_unary_op(
        &self,
        node: &Ast,
        op: &UnaryOp,
        input: Value,
        frame: &Frame,
    ) -> Result<Value> {
        match *op {
            UnaryOp::Minus(ref value) => {
                let result = self.evaluate(value, input, frame)?;
                match *result {
                    ValueKind::Undefined => Ok(value::UNDEFINED),
                    ValueKind::Number(num) if !num.is_nan() => Ok(Value::number(-num)),
                    _ => Err(Error::D1002NegatingNonNumeric(
                        node.char_index,
                        result.dump(),
                    )),
                }
            }
            UnaryOp::ArrayConstructor(ref array) => {
                let mut result = Value::array(if node.cons_array {
                    ArrayFlags::CONS
                } else {
                    ArrayFlags::empty()
                });
                for item in array.iter() {
                    let value = self.evaluate(item, input, frame)?;
                    if let AstKind::Unary(UnaryOp::ArrayConstructor(..)) = item.kind {
                        result.push(value);
                    } else {
                        result = fn_append(
                            &self.fn_context("append", node.char_index, input, frame),
                            result,
                            value,
                        )?;
                    }
                }
                Ok(result)
            }
            UnaryOp::ObjectConstructor(ref object) => {
                self.evaluate_group_expression(node.char_index, object, input, frame)
            }
        }
    }

    fn evaluate_group_expression(
        &self,
        char_index: usize,
        object: &[(Ast, Ast)],
        input: Value,
        frame: &Frame,
    ) -> Result<Value> {
        struct Group {
            pub data: Value,
            pub index: usize,
        }

        let mut groups: HashMap<String, Group> = HashMap::new();
        let input = input.wrap_in_array_if_needed(ArrayFlags::empty());

        if input.is_empty() {
            input.push_new(ValueKind::Undefined);
        }

        for item in input.members() {
            for (index, pair) in object.iter().enumerate() {
                let key = self.evaluate(&pair.0, *item, frame)?;
                if !key.is_string() {
                    return Err(Error::T1003NonStringKey(char_index, key.dump()));
                }

                let key = key.as_str();

                match groups.entry(key.to_string()) {
                    hash_map::Entry::Occupied(mut entry) => {
                        let group = entry.get_mut();
                        if group.index != index {
                            return Err(Error::D1009MultipleKeys(char_index, key.to_string()));
                        }
                        group.data = fn_append(
                            &self.fn_context("append", char_index, input, frame),
                            group.data,
                            *item,
                        )?;
                    }
                    hash_map::Entry::Vacant(entry) => {
                        entry.insert(Group { data: *item, index });
                    }
                };
            }
        }

        let result = Value::object();

        for key in groups.keys() {
            let group = groups.get(key).unwrap();
            let value = self.evaluate(&object[group.index].1, group.data, frame)?;
            if !value.is_undefined() {
                result.insert(key, value);
            }
        }

        Ok(result)
    }

    fn evaluate_binary_op(
        &self,
        node: &Ast,
        op: &BinaryOp,
        lhs_ast: &Ast,
        rhs_ast: &Ast,
        input: Value,
        frame: &Frame,
    ) -> Result<Value> {
        if *op == BinaryOp::Bind {
            if let AstKind::Var(ref name) = lhs_ast.kind {
                let rhs = self.evaluate(rhs_ast, input, frame)?;
                frame.bind(name, rhs);
                return Ok(rhs);
            }
            unreachable!()
        }

        // NOTE: rhs is not evaluated until absolutely necessary to support short circuiting
        // of boolean expressions.
        let lhs = self.evaluate(lhs_ast, input, frame)?;

        match op {
            BinaryOp::Add
            | BinaryOp::Subtract
            | BinaryOp::Multiply
            | BinaryOp::Divide
            | BinaryOp::Modulus => {
                let rhs = self.evaluate(rhs_ast, input, frame)?;

                let lhs = match *lhs {
                    ValueKind::Undefined => return Ok(value::UNDEFINED),
                    ValueKind::Number(n) if !n.is_nan() => f64::from(n),
                    _ => {
                        return Err(Error::T2001LeftSideNotNumber(
                            node.char_index,
                            op.to_string(),
                        ))
                    }
                };

                let rhs = match *rhs {
                    ValueKind::Undefined => return Ok(value::UNDEFINED),
                    ValueKind::Number(n) if !n.is_nan() => f64::from(n),
                    _ => {
                        return Err(Error::T2002RightSideNotNumber(
                            node.char_index,
                            op.to_string(),
                        ))
                    }
                };

                let result = match op {
                    BinaryOp::Add => lhs + rhs,
                    BinaryOp::Subtract => lhs - rhs,
                    BinaryOp::Multiply => lhs * rhs,
                    BinaryOp::Divide => lhs / rhs,
                    BinaryOp::Modulus => lhs % rhs,
                    _ => unreachable!(),
                };

                if result.is_infinite() {
                    Err(Error::D1001NumberOfOutRange(result))
                } else {
                    Ok(Value::number(result))
                }
            }

            BinaryOp::LessThan
            | BinaryOp::LessThanEqual
            | BinaryOp::GreaterThan
            | BinaryOp::GreaterThanEqual => {
                let rhs = self.evaluate(rhs_ast, input, frame)?;

                if lhs.is_undefined() || rhs.is_undefined() {
                    return Ok(value::UNDEFINED);
                }

                if !((lhs.is_number() || lhs.is_string()) && (rhs.is_number() || rhs.is_string())) {
                    return Err(Error::T2010BinaryOpTypes(node.char_index, op.to_string()));
                }

                if let (ValueKind::Number(ref lhs), ValueKind::Number(ref rhs)) = (&*lhs, &*rhs) {
                    let lhs = f64::from(*lhs);
                    let rhs = f64::from(*rhs);
                    return Ok(Value::bool(match op {
                        BinaryOp::LessThan => lhs < rhs,
                        BinaryOp::LessThanEqual => lhs <= rhs,
                        BinaryOp::GreaterThan => lhs > rhs,
                        BinaryOp::GreaterThanEqual => lhs >= rhs,
                        _ => unreachable!(),
                    }));
                }

                if let (ValueKind::String(ref lhs), ValueKind::String(ref rhs)) = (&*lhs, &*rhs) {
                    return Ok(Value::bool(match op {
                        BinaryOp::LessThan => lhs < rhs,
                        BinaryOp::LessThanEqual => lhs <= rhs,
                        BinaryOp::GreaterThan => lhs > rhs,
                        BinaryOp::GreaterThanEqual => lhs >= rhs,
                        _ => unreachable!(),
                    }));
                }

                Err(Error::T2009BinaryOpMismatch(
                    node.char_index,
                    lhs.dump(),
                    rhs.dump(),
                    op.to_string(),
                ))
            }

            BinaryOp::Equal | BinaryOp::NotEqual => {
                let rhs = self.evaluate(rhs_ast, input, frame)?;

                if lhs.is_undefined() || rhs.is_undefined() {
                    return Ok(Value::bool(false));
                }

                Ok(Value::bool(match op {
                    BinaryOp::Equal => lhs == rhs,
                    BinaryOp::NotEqual => lhs != rhs,
                    _ => unreachable!(),
                }))
            }

            BinaryOp::Range => {
                let rhs = self.evaluate(rhs_ast, input, frame)?;

                if !lhs.is_undefined() && !lhs.is_integer() {
                    return Err(Error::T2003LeftSideNotInteger(node.char_index));
                };

                if !rhs.is_undefined() && !rhs.is_integer() {
                    return Err(Error::T2004RightSideNotInteger(node.char_index));
                }

                if lhs.is_undefined() || rhs.is_undefined() {
                    return Ok(value::UNDEFINED);
                }

                let lhs = lhs.as_usize();
                let rhs = rhs.as_usize();

                if lhs > rhs {
                    return Ok(value::UNDEFINED);
                }

                let size = rhs - lhs + 1;
                if size > 10_000_000 {
                    // TODO: D2014
                    unreachable!()
                }

                let result = Value::array_with_capacity(size, ArrayFlags::SEQUENCE);
                for index in lhs..rhs + 1 {
                    result.push_new(ValueKind::Number(index.into()));
                }

                Ok(result)
            }

            BinaryOp::Concat => {
                let rhs = self.evaluate(rhs_ast, input, frame)?;
                let mut result = String::new();
                if !lhs.is_undefined() {
                    result.push_str(
                        &fn_string(
                            &self.fn_context("string", node.char_index, input, frame),
                            lhs,
                        )?
                        .as_str(),
                    );
                }
                if !rhs.is_undefined() {
                    result.push_str(
                        &fn_string(
                            &self.fn_context("string", node.char_index, input, frame),
                            rhs,
                        )?
                        .as_str(),
                    );
                }
                Ok(Value::string(result))
            }

            BinaryOp::And => Ok(Value::bool(
                lhs.is_truthy() && self.evaluate(rhs_ast, input, frame)?.is_truthy(),
            )),

            BinaryOp::Or => Ok(Value::bool(
                lhs.is_truthy() || self.evaluate(rhs_ast, input, frame)?.is_truthy(),
            )),

            BinaryOp::Apply => {
                if let AstKind::Function {
                    ref proc,
                    ref args,
                    is_partial,
                    ..
                } = rhs_ast.kind
                {
                    // Function invocation with lhs as the first argument
                    self.evaluate_function(input, proc, args, is_partial, frame, Some(&lhs))
                } else {
                    let rhs = self.evaluate(rhs_ast, input, frame)?;

                    if !rhs.is_function() {
                        // TODO T2006
                        unreachable!()
                    }

                    if lhs.is_function() {
                        // Apply function chaining
                        let chain = self.evaluate(&self.chain_ast, value::UNDEFINED, frame)?;

                        let args = Value::array_with_capacity(2, ArrayFlags::empty());
                        args.push(lhs);
                        args.push(rhs);

                        self.apply_function(
                            lhs_ast.char_index,
                            value::UNDEFINED,
                            chain,
                            args,
                            frame,
                        )
                    } else {
                        let args = Value::array_with_capacity(1, ArrayFlags::empty());
                        args.push(lhs);
                        self.apply_function(rhs_ast.char_index, value::UNDEFINED, rhs, args, frame)
                    }
                }
            }

            BinaryOp::In => {
                let rhs = self.evaluate(rhs_ast, input, frame)?;

                if lhs.is_undefined() || rhs.is_undefined() {
                    return Ok(Value::bool(false));
                }

                let rhs = rhs.wrap_in_array_if_needed(ArrayFlags::empty());

                for item in rhs.members() {
                    if *item == lhs {
                        return Ok(Value::bool(true));
                    }
                }

                Ok(Value::bool(false))
            }

            _ => unimplemented!("TODO: binary op not supported yet: {:#?}", *op),
        }
    }

    fn evaluate_ternary(
        &self,
        cond: &Ast,
        truthy: &Ast,
        falsy: Option<&Ast>,
        input: Value,
        frame: &Frame,
    ) -> Result<Value> {
        let cond = self.evaluate(cond, input, frame)?;
        if cond.is_truthy() {
            self.evaluate(truthy, input, frame)
        } else if let Some(falsy) = falsy {
            self.evaluate(falsy, input, frame)
        } else {
            Ok(value::UNDEFINED)
        }
    }

    fn evaluate_path(
        &self,
        node: &Ast,
        steps: &[Ast],
        input: Value,
        frame: &Frame,
    ) -> Result<Value> {
        let mut input = if input.is_array() && !matches!(steps[0].kind, AstKind::Var(..)) {
            input
        } else {
            input.wrap_in_array(ArrayFlags::empty())
        };

        let mut result = value::UNDEFINED;

        for (index, step) in steps.iter().enumerate() {
            result = if index == 0 && step.cons_array {
                self.evaluate(step, input, frame)?
            } else {
                self.evaluate_step(step, input, frame, index == steps.len() - 1)?
            };

            if result.is_undefined() || (result.is_array() && result.is_empty()) {
                break;
            }

            // if step.focus.is_none() {
            input = result
            // }
        }

        if node.keep_singleton_array {
            let mut flags = result.get_flags();
            if flags.contains(ArrayFlags::CONS) && !flags.contains(ArrayFlags::SEQUENCE) {
                result = result.wrap_in_array(flags | ArrayFlags::SEQUENCE);
            }
            flags |= ArrayFlags::SINGLETON;
            result.set_flags(flags);
        }

        if let Some((char_index, ref object)) = node.group_by {
            result = self.evaluate_group_expression(char_index, object, result, frame)?;
        }

        Ok(result)
    }

    fn evaluate_step(
        &self,
        step: &Ast,
        input: Value,
        frame: &Frame,
        last_step: bool,
    ) -> Result<Value> {
        let mut result = Value::array(ArrayFlags::SEQUENCE);

        if let AstKind::Sort(ref sorts) = step.kind {
            result = self.evaluate_sorts(sorts, input, frame)?;
            if let Some(ref stages) = step.stages {
                result = self.evaluate_stages(stages, result, frame)?;
            }
            return Ok(result);
        }

        for item in input.members() {
            let mut item_result = self.evaluate(step, *item, frame)?;

            if let Some(ref stages) = step.stages {
                for stage in stages {
                    item_result = self.evaluate_filter(stage, item_result, frame)?;
                }
            }

            if !item_result.is_undefined() {
                result.push(item_result);
            }
        }

        Ok(
            if last_step
                && result.len() == 1
                && result.get_member(0).is_array()
                && !result.get_member(0).has_flags(ArrayFlags::SEQUENCE)
            {
                result.get_member(0)
            } else {
                let result_sequence = Value::array(ArrayFlags::SEQUENCE);

                for result_item in result.members() {
                    if !result_item.is_array() || result_item.has_flags(ArrayFlags::CONS) {
                        result_sequence.push(*result_item);
                    } else {
                        for item in result_item.members() {
                            result_sequence.push(*item);
                        }
                    }
                }
                result_sequence
            },
        )
    }

    fn evaluate_sorts(
        &self,
        _sorts: &[(Ast, bool)],
        _inputt: Value,
        _frame: &Frame,
    ) -> Result<Value> {
        unimplemented!("Sorts not yet implemented")
    }

    fn evaluate_stages(&self, _stages: &[Ast], _input: Value, _frame: &Frame) -> Result<Value> {
        unimplemented!("Stages not yet implemented")
    }

    fn evaluate_filter(&self, node: &Ast, input: Value, frame: &Frame) -> Result<Value> {
        let mut result = Value::array(ArrayFlags::SEQUENCE);
        let input = input.wrap_in_array_if_needed(ArrayFlags::empty());

        let get_index = |n: f64| {
            let mut index = n.floor() as isize;
            let length = if input.is_array() {
                input.len() as isize
            } else {
                1
            };
            if index < 0 {
                // Count from the end of the array
                index += length;
            }
            index as usize
        };

        match node.kind {
            AstKind::Filter(ref filter) => match filter.kind {
                AstKind::Number(n) => {
                    let index = get_index(n.into());
                    let item = input.get_member(index as usize);
                    if !item.is_undefined() {
                        if item.is_array() {
                            result = item;
                        } else {
                            result.push(item);
                        }
                    }
                }
                _ => {
                    for (i, item) in input.members().enumerate() {
                        let mut index = self.evaluate(filter, *item, frame)?;
                        if index.is_number() && !index.is_nan() {
                            index = index.wrap_in_array(ArrayFlags::empty());
                        }
                        if index.is_array() && index.members().all(|v| v.is_number() && !v.is_nan())
                        {
                            index.members().for_each(|v| {
                                let index = get_index(v.as_f64());
                                if index == i {
                                    result.push(*item);
                                }
                            });
                        } else if index.is_truthy() {
                            result.push(*item);
                        }
                    }
                }
            },
            _ => unimplemented!("Filters other than numbers are not yet supported"),
        };

        Ok(result)
    }

    pub fn evaluate_function(
        &self,
        input: Value,
        proc: &Ast,
        args: &[Ast],
        _is_partial: bool,
        frame: &Frame,
        context: Option<&Value>,
    ) -> Result<Value> {
        let evaluated_proc = self.evaluate(proc, input, frame)?;

        // Help the user out if they forgot a '$'
        if evaluated_proc.is_undefined() {
            if let AstKind::Path(ref steps) = proc.kind {
                if let AstKind::Name(ref name) = steps[0].kind {
                    if frame.lookup(name).is_some() {
                        return Err(Error::T1005InvokedNonFunctionSuggest(
                            proc.char_index,
                            name.clone(),
                        ));
                    }
                }
            }
        }

        let evaluated_args = Value::array_with_capacity(args.len(), ArrayFlags::empty());

        if let Some(context) = context {
            evaluated_args.push(*context);
        }

        for arg in args {
            let arg = self.evaluate(arg, input, frame)?;
            evaluated_args.push(arg);
        }

        let mut result = self.apply_function(
            proc.char_index,
            input,
            evaluated_proc,
            evaluated_args,
            frame,
        )?;

        // Trampoline loop for tail-call optimization
        // TODO: This loop needs help
        while let ValueKind::Lambda {
            ast,
            input: ref lambda_input,
            frame: ref lambda_frame,
            ..
        } = *result
        {
            if let AstKind::Lambda {
                body, thunk: true, ..
            } = unsafe { &(*ast).kind }
            {
                if let AstKind::Function {
                    ref proc, ref args, ..
                } = body.kind
                {
                    let next = self.evaluate(proc, *lambda_input, lambda_frame)?;
                    let evaluated_args =
                        Value::array_with_capacity(args.len(), ArrayFlags::empty());

                    for arg in args {
                        let arg = self.evaluate(arg, *lambda_input, lambda_frame)?;
                        evaluated_args.push(arg);
                    }

                    result =
                        self.apply_function(proc.char_index, input, next, evaluated_args, frame)?;
                } else {
                    unreachable!()
                }
            } else {
                break;
            }
        }

        Ok(result)
    }

    pub fn apply_function(
        &self,
        char_index: usize,
        input: Value,
        evaluated_proc: Value,
        evaluated_args: Value,
        frame: &Frame,
    ) -> Result<Value> {
        match *evaluated_proc {
            ValueKind::Lambda {
                ast,
                ref frame,
                ref input,
                ..
            } => {
                if let AstKind::Lambda { body, args, .. } = unsafe { &(*ast).kind } {
                    // Create a new frame for use in the lambda, so it can have locals
                    let frame = Frame::new_with_parent(frame);

                    // Bind the arguments to their respective names
                    for (index, arg) in args.iter().enumerate() {
                        if let AstKind::Var(ref name) = arg.kind {
                            frame.bind(name, evaluated_args.get_member(index));
                        } else {
                            unreachable!()
                        }
                    }

                    // Evaluate the lambda!
                    self.evaluate(body, *input, &frame)
                } else {
                    unreachable!()
                }
            }
            ValueKind::NativeFn0(ref name, ref func) => {
                func(&self.fn_context(name, char_index, input, frame))
            }
            ValueKind::NativeFn1(ref name, ref func) => {
                let context = self.fn_context(name, char_index, input, frame);
                if evaluated_args.len() > 1 {
                    Err(Error::T0410ArgumentNotValid(
                        context.char_index,
                        2,
                        context.name.to_string(),
                    ))
                } else if evaluated_args.is_empty() {
                    // Some functions take the input as the first argument if one was not provided
                    func(&context, input)
                } else {
                    func(&context, evaluated_args.get_member(0))
                }
            }
            ValueKind::NativeFn2(ref name, ref func) => {
                let context = self.fn_context(name, char_index, input, frame);
                if evaluated_args.len() > 2 {
                    Err(Error::T0410ArgumentNotValid(
                        context.char_index,
                        3,
                        context.name.to_string(),
                    ))
                } else {
                    func(
                        &context,
                        evaluated_args.get_member(0),
                        evaluated_args.get_member(1),
                    )
                }
            }
            ValueKind::NativeFn3(ref name, ref func) => {
                let context = self.fn_context(name, char_index, input, frame);
                if evaluated_args.len() > 3 {
                    Err(Error::T0410ArgumentNotValid(
                        context.char_index,
                        4,
                        context.name.to_string(),
                    ))
                } else {
                    func(
                        &context,
                        evaluated_args.get_member(0),
                        evaluated_args.get_member(1),
                        evaluated_args.get_member(2),
                    )
                }
            }
            _ => Err(Error::T1006InvokedNonFunction(char_index)),
        }
    }
}
