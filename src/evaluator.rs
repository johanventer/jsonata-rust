use bumpalo::Bump;
use std::cell::RefCell;
use std::collections::{hash_map, HashMap};
use std::time::Instant;

use crate::{Error, Result};

use super::ast::*;
use super::frame::Frame;
use super::functions::*;
use super::value::{ArrayFlags, Value};

struct EvaluatorInternal {
    depth: usize,
    started_at: Option<Instant>,
    max_depth: Option<usize>,
    time_limit: Option<usize>,
}

pub struct Evaluator<'a> {
    chain_ast: Option<Ast>,
    arena: &'a Bump,
    internal: RefCell<EvaluatorInternal>,
}

impl<'a> Evaluator<'a> {
    pub fn new(
        chain_ast: Option<Ast>,
        arena: &'a Bump,
        max_depth: Option<usize>,
        time_limit: Option<usize>,
    ) -> Self {
        Evaluator {
            chain_ast,
            arena,
            internal: RefCell::new(EvaluatorInternal {
                depth: 0,
                started_at: None,
                max_depth,
                time_limit,
            }),
        }
    }

    fn fn_context<'e>(
        &'e self,
        name: &'a str,
        char_index: usize,
        input: &'a Value<'a>,
        frame: &Frame<'a>,
    ) -> FunctionContext<'a, 'e> {
        FunctionContext {
            name,
            char_index,
            input,
            frame: frame.clone(),
            arena: self.arena,
            evaluator: self,
        }
    }

    fn check_limits(&self, inc_or_dec: bool) -> Result<()> {
        let mut internal = self.internal.borrow_mut();
        internal.depth = if inc_or_dec {
            internal.depth + 1
        } else {
            internal.depth - 1
        };
        if let Some(started_at) = internal.started_at {
            if let Some(time_limit) = internal.time_limit {
                if started_at.elapsed().as_millis() >= time_limit as u128 {
                    return Err(Error::U1001Timeout);
                }
            }
        } else {
            internal.started_at = Some(Instant::now());
        }
        if let Some(max_depth) = internal.max_depth {
            if internal.depth > max_depth {
                return Err(Error::U1001StackOverflow);
            }
        }
        Ok(())
    }

    pub fn evaluate(
        &self,
        node: &Ast,
        input: &'a Value<'a>,
        frame: &Frame<'a>,
    ) -> Result<&'a Value<'a>> {
        self.check_limits(true)?;

        let mut result = match node.kind {
            AstKind::Null => Value::null(self.arena),
            AstKind::Bool(b) => Value::bool(self.arena, b),
            AstKind::String(ref s) => Value::string(self.arena, String::from(s)),
            AstKind::Number(n) => Value::number(self.arena, n),
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
                self.fn_context("lookup", node.char_index, input, frame),
                input,
                name,
            ),
            AstKind::Lambda { .. } => Value::lambda(self.arena, node, input, frame.clone()),
            AstKind::Function {
                ref proc,
                ref args,
                is_partial,
                ..
            } => self.evaluate_function(input, proc, args, is_partial, frame, None)?,
            AstKind::Wildcard => self.evaluate_wildcard(node, input, frame)?,
            AstKind::Descendent => self.evaluate_descendants(input)?,
            _ => unimplemented!("TODO: node kind not yet supported: {:#?}", node.kind),
        };

        if let Some(filters) = &node.predicates {
            for filter in filters {
                result = self.evaluate_filter(filter, result, frame)?
            }
        }

        self.check_limits(false)?;

        Ok(if result.has_flags(ArrayFlags::SEQUENCE) {
            if node.keep_array {
                result = result
                    .clone_array_with_flags(self.arena, result.get_flags() | ArrayFlags::SINGLETON)
            }
            if result.is_empty() {
                Value::undefined()
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

    fn evaluate_block(
        &self,
        exprs: &[Ast],
        input: &'a Value<'a>,
        frame: &Frame<'a>,
    ) -> Result<&'a Value<'a>> {
        let frame = Frame::new_with_parent(frame);
        if exprs.is_empty() {
            return Ok(Value::undefined());
        }

        let mut result = Value::undefined();
        for expr in exprs {
            result = self.evaluate(expr, input, &frame)?;
        }

        Ok(result)
    }

    fn evaluate_var(
        &self,
        name: &str,
        input: &'a Value<'a>,
        frame: &Frame<'a>,
    ) -> Result<&'a Value<'a>> {
        Ok(if name.is_empty() {
            if input.has_flags(ArrayFlags::WRAPPED) {
                input.get_member(0)
            } else {
                input
            }
        } else if let Some(value) = frame.lookup(name) {
            value
        } else {
            Value::undefined()
        })
    }

    fn evaluate_unary_op(
        &self,
        node: &Ast,
        op: &UnaryOp,
        input: &'a Value<'a>,
        frame: &Frame<'a>,
    ) -> Result<&'a Value<'a>> {
        match *op {
            UnaryOp::Minus(ref value) => {
                let result = self.evaluate(value, input, frame)?;
                match result {
                    Value::Undefined => Ok(Value::undefined()),
                    Value::Number(n) if result.is_valid_number()? => {
                        Ok(Value::number(self.arena, -n))
                    }
                    _ => Err(Error::D1002NegatingNonNumeric(
                        node.char_index,
                        result.to_string(),
                    )),
                }
            }
            UnaryOp::ArrayConstructor(ref array) => {
                let mut result = Value::array(
                    self.arena,
                    if node.cons_array {
                        ArrayFlags::CONS
                    } else {
                        ArrayFlags::empty()
                    },
                );
                for item in array.iter() {
                    let value = self.evaluate(item, input, frame)?;
                    if let AstKind::Unary(UnaryOp::ArrayConstructor(..)) = item.kind {
                        result.push(value);
                    } else {
                        result = fn_append_internal(
                            self.fn_context("append", node.char_index, input, frame),
                            result,
                            value,
                        );
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
        input: &'a Value<'a>,
        frame: &Frame<'a>,
    ) -> Result<&'a Value<'a>> {
        struct Group<'a> {
            pub data: &'a Value<'a>,
            pub index: usize,
        }

        let mut groups: HashMap<String, Group> = HashMap::new();

        let input = if input.is_array() && input.is_empty() {
            let input = Value::array_with_capacity(self.arena, 1, ArrayFlags::empty());
            input.push(Value::undefined());
            input
        } else if !input.is_array() {
            let wrapped = Value::array_with_capacity(self.arena, 1, ArrayFlags::empty());
            wrapped.push(input);
            wrapped
        } else {
            input
        };

        for item in input.members() {
            for (index, pair) in object.iter().enumerate() {
                let key = self.evaluate(&pair.0, item, frame)?;
                if !key.is_string() {
                    return Err(Error::T1003NonStringKey(char_index, key.to_string()));
                }

                let key = key.as_str();

                match groups.entry(key.to_string()) {
                    hash_map::Entry::Occupied(mut entry) => {
                        let group = entry.get_mut();
                        if group.index != index {
                            return Err(Error::D1009MultipleKeys(char_index, key.to_string()));
                        }
                        let args = Value::array_with_capacity(self.arena, 2, ArrayFlags::empty());
                        args.push(group.data);
                        args.push(item);
                        group.data =
                            fn_append(self.fn_context("append", char_index, input, frame), args)?;
                    }
                    hash_map::Entry::Vacant(entry) => {
                        entry.insert(Group { data: item, index });
                    }
                };
            }
        }

        let result = Value::object(self.arena);

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
        input: &'a Value<'a>,
        frame: &Frame<'a>,
    ) -> Result<&'a Value<'a>> {
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

                let lhs = if lhs.is_undefined() {
                    return Ok(Value::undefined());
                } else if lhs.is_valid_number()? {
                    lhs.as_f64()
                } else {
                    return Err(Error::T2001LeftSideNotNumber(
                        node.char_index,
                        op.to_string(),
                    ));
                };

                let rhs = if rhs.is_undefined() {
                    return Ok(Value::undefined());
                } else if rhs.is_valid_number()? {
                    rhs.as_f64()
                } else {
                    return Err(Error::T2002RightSideNotNumber(
                        node.char_index,
                        op.to_string(),
                    ));
                };

                let result = match op {
                    BinaryOp::Add => lhs + rhs,
                    BinaryOp::Subtract => lhs - rhs,
                    BinaryOp::Multiply => lhs * rhs,
                    BinaryOp::Divide => lhs / rhs,
                    BinaryOp::Modulus => lhs % rhs,
                    _ => unreachable!(),
                };

                Ok(Value::number(self.arena, result))
            }

            BinaryOp::LessThan
            | BinaryOp::LessThanEqual
            | BinaryOp::GreaterThan
            | BinaryOp::GreaterThanEqual => {
                let rhs = self.evaluate(rhs_ast, input, frame)?;

                if lhs.is_undefined() || rhs.is_undefined() {
                    return Ok(Value::undefined());
                }

                if !((lhs.is_number() || lhs.is_string()) && (rhs.is_number() || rhs.is_string())) {
                    return Err(Error::T2010BinaryOpTypes(node.char_index, op.to_string()));
                }

                if lhs.is_number() && rhs.is_number() {
                    let lhs = lhs.as_f64();
                    let rhs = rhs.as_f64();
                    return Ok(Value::bool(
                        self.arena,
                        match op {
                            BinaryOp::LessThan => lhs < rhs,
                            BinaryOp::LessThanEqual => lhs <= rhs,
                            BinaryOp::GreaterThan => lhs > rhs,
                            BinaryOp::GreaterThanEqual => lhs >= rhs,
                            _ => unreachable!(),
                        },
                    ));
                }

                if let (Value::String(ref lhs), Value::String(ref rhs)) = (lhs, rhs) {
                    return Ok(Value::bool(
                        self.arena,
                        match op {
                            BinaryOp::LessThan => lhs < rhs,
                            BinaryOp::LessThanEqual => lhs <= rhs,
                            BinaryOp::GreaterThan => lhs > rhs,
                            BinaryOp::GreaterThanEqual => lhs >= rhs,
                            _ => unreachable!(),
                        },
                    ));
                }

                Err(Error::T2009BinaryOpMismatch(
                    node.char_index,
                    lhs.to_string(),
                    rhs.to_string(),
                    op.to_string(),
                ))
            }

            BinaryOp::Equal | BinaryOp::NotEqual => {
                let rhs = self.evaluate(rhs_ast, input, frame)?;

                if lhs.is_undefined() || rhs.is_undefined() {
                    return Ok(Value::bool(self.arena, false));
                }

                Ok(Value::bool(
                    self.arena,
                    match op {
                        BinaryOp::Equal => lhs == rhs,
                        BinaryOp::NotEqual => lhs != rhs,
                        _ => unreachable!(),
                    },
                ))
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
                    return Ok(Value::undefined());
                }

                let lhs = lhs.as_isize();
                let rhs = rhs.as_isize();

                if lhs > rhs {
                    return Ok(Value::undefined());
                }

                let size = rhs - lhs + 1;
                if size > 10_000_000 {
                    return Err(Error::D2014RangeOutOfBounds(node.char_index, size));
                }

                Ok(Value::range(self.arena, lhs, rhs))
            }

            BinaryOp::Concat => {
                let rhs = self.evaluate(rhs_ast, input, frame)?;
                let mut result = String::new();
                if !lhs.is_undefined() {
                    result.push_str(
                        &fn_string(
                            self.fn_context("string", node.char_index, input, frame),
                            Value::wrap_in_array(self.arena, lhs, ArrayFlags::empty()),
                        )?
                        .as_str(),
                    );
                }
                if !rhs.is_undefined() {
                    result.push_str(
                        &fn_string(
                            self.fn_context("string", node.char_index, input, frame),
                            Value::wrap_in_array(self.arena, rhs, ArrayFlags::empty()),
                        )?
                        .as_str(),
                    );
                }
                Ok(Value::string(self.arena, result))
            }

            BinaryOp::And => Ok(Value::bool(
                self.arena,
                lhs.is_truthy() && self.evaluate(rhs_ast, input, frame)?.is_truthy(),
            )),

            BinaryOp::Or => Ok(Value::bool(
                self.arena,
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
                    Ok(self.evaluate_function(input, proc, args, is_partial, frame, Some(lhs))?)
                } else {
                    let rhs = self.evaluate(rhs_ast, input, frame)?;

                    if !rhs.is_function() {
                        // TODO T2006
                        unreachable!()
                    }

                    if lhs.is_function() {
                        // Apply function chaining
                        let chain = self.evaluate(
                            self.chain_ast.as_ref().unwrap(),
                            Value::undefined(),
                            frame,
                        )?;

                        let args = Value::array_with_capacity(self.arena, 2, ArrayFlags::empty());
                        args.push(lhs);
                        args.push(rhs);

                        Ok(self.apply_function(
                            lhs_ast.char_index,
                            Value::undefined(),
                            chain,
                            args,
                            frame,
                        )?)
                    } else {
                        let args = Value::array_with_capacity(self.arena, 1, ArrayFlags::empty());
                        args.push(lhs);
                        Ok(self.apply_function(
                            rhs_ast.char_index,
                            Value::undefined(),
                            rhs,
                            args,
                            frame,
                        )?)
                    }
                }
            }

            BinaryOp::In => {
                let rhs = self.evaluate(rhs_ast, input, frame)?;

                if lhs.is_undefined() || rhs.is_undefined() {
                    return Ok(Value::bool(self.arena, false));
                }

                let rhs = Value::wrap_in_array_if_needed(self.arena, rhs, ArrayFlags::empty());

                for item in rhs.members() {
                    if item == lhs {
                        return Ok(Value::bool(self.arena, true));
                    }
                }

                Ok(Value::bool(self.arena, false))
            }

            _ => unimplemented!("TODO: binary op not supported yet: {:#?}", *op),
        }
    }

    fn evaluate_ternary(
        &self,
        cond: &Ast,
        truthy: &Ast,
        falsy: Option<&Ast>,
        input: &'a Value<'a>,
        frame: &Frame<'a>,
    ) -> Result<&'a Value<'a>> {
        let cond = self.evaluate(cond, input, frame)?;
        if cond.is_truthy() {
            self.evaluate(truthy, input, frame)
        } else if let Some(falsy) = falsy {
            self.evaluate(falsy, input, frame)
        } else {
            Ok(Value::undefined())
        }
    }

    fn evaluate_path(
        &self,
        node: &Ast,
        steps: &[Ast],
        input: &'a Value<'a>,
        frame: &Frame<'a>,
    ) -> Result<&'a Value<'a>> {
        let mut input = if input.is_array() && !matches!(steps[0].kind, AstKind::Var(..)) {
            input
        } else {
            Value::wrap_in_array(self.arena, input, ArrayFlags::empty())
        };

        let mut result = Value::undefined();

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
            let flags = result.get_flags();
            if flags.contains(ArrayFlags::CONS) && !flags.contains(ArrayFlags::SEQUENCE) {
                result = Value::wrap_in_array(
                    self.arena,
                    result,
                    flags | ArrayFlags::SEQUENCE | ArrayFlags::SINGLETON,
                );
            }
            result = result.clone_array_with_flags(self.arena, flags | ArrayFlags::SINGLETON);
        }

        if let Some((char_index, ref object)) = node.group_by {
            self.evaluate_group_expression(char_index, object, result, frame)
        } else {
            Ok(result)
        }
    }

    fn evaluate_step(
        &self,
        step: &Ast,
        input: &'a Value<'a>,
        frame: &Frame<'a>,
        last_step: bool,
    ) -> Result<&'a Value<'a>> {
        if let AstKind::Sort(ref sorts) = step.kind {
            let mut result = self.evaluate_sorts(sorts, input, frame)?;
            if let Some(ref stages) = step.stages {
                result = self.evaluate_stages(stages, result, frame)?;
            }
            return Ok(result);
        }

        let result = Value::array(self.arena, ArrayFlags::SEQUENCE);

        for item in input.members() {
            let mut item_result = self.evaluate(step, item, frame)?;

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
                let result_sequence = Value::array(self.arena, ArrayFlags::SEQUENCE);

                for result_item in result.members() {
                    if !result_item.is_array() || result_item.has_flags(ArrayFlags::CONS) {
                        result_sequence.push(result_item);
                    } else {
                        for item in result_item.members() {
                            result_sequence.push(item);
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
        _inputt: &'a Value<'a>,
        _frame: &Frame<'a>,
    ) -> Result<&'a Value<'a>> {
        unimplemented!("Sorts not yet implemented")
    }

    fn evaluate_stages(
        &self,
        _stages: &[Ast],
        _input: &'a Value<'a>,
        _frame: &Frame<'a>,
    ) -> Result<&'a Value<'a>> {
        unimplemented!("Stages not yet implemented")
    }

    fn evaluate_filter(
        &self,
        node: &Ast,
        input: &'a Value<'a>,
        frame: &Frame<'a>,
    ) -> Result<&'a Value<'a>> {
        let result = Value::array(self.arena, ArrayFlags::SEQUENCE);
        let input = Value::wrap_in_array_if_needed(self.arena, input, ArrayFlags::empty());

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
                    let index = get_index(n);
                    let item = input.get_member(index as usize);
                    if !item.is_undefined() {
                        if item.is_array() {
                            return Ok(item);
                        } else {
                            result.push(item);
                        }
                    }
                }
                _ => {
                    for (i, item) in input.members().enumerate() {
                        let mut index = self.evaluate(filter, item, frame)?;
                        if index.is_valid_number()? {
                            index = Value::wrap_in_array(self.arena, index, ArrayFlags::empty());
                        }
                        if index.is_array_of_valid_numbers()? {
                            index.members().for_each(|v| {
                                let index = get_index(v.as_f64());
                                if index == i {
                                    result.push(item);
                                }
                            });
                        } else if index.is_truthy() {
                            result.push(item);
                        }
                    }
                }
            },
            _ => unimplemented!("Filters other than numbers are not yet supported"),
        };

        Ok(result)
    }

    fn evaluate_wildcard(
        &self,
        node: &Ast,
        input: &'a Value<'a>,
        frame: &Frame<'a>,
    ) -> Result<&'a Value<'a>> {
        let mut result = Value::array(self.arena, ArrayFlags::SEQUENCE);

        let input = if input.is_array() && input.has_flags(ArrayFlags::WRAPPED) && !input.is_empty()
        {
            input.get_member(0)
        } else {
            input
        };

        if input.is_object() {
            for (_key, value) in input.entries() {
                if value.is_array() {
                    let value = value.flatten(self.arena);
                    result = fn_append_internal(
                        self.fn_context("append", node.char_index, input, frame),
                        result,
                        value,
                    );
                } else {
                    result.push(value)
                }
            }
        }

        Ok(result)
    }

    fn evaluate_descendants(&self, input: &'a Value<'a>) -> Result<&'a Value<'a>> {
        Ok(if !input.is_undefined() {
            let result_sequence =
                self.recurse_descendants(input, Value::array(self.arena, ArrayFlags::SEQUENCE));

            if result_sequence.len() == 1 {
                result_sequence.get_member(0)
            } else {
                result_sequence
            }
        } else {
            input
        })
    }

    fn recurse_descendants(
        &self,
        input: &'a Value<'a>,
        result_sequence: &'a mut Value<'a>,
    ) -> &'a mut Value<'a> {
        if !input.is_array() {
            result_sequence.push(input);
        }

        let mut result_sequence = result_sequence;

        if input.is_array() {
            for member in input.members() {
                result_sequence = self.recurse_descendants(member, result_sequence);
            }
        } else if input.is_object() {
            for (_key, value) in input.entries() {
                result_sequence = self.recurse_descendants(value, result_sequence);
            }
        }

        result_sequence
    }

    fn evaluate_function(
        &self,
        input: &'a Value<'a>,
        proc: &Ast,
        args: &[Ast],
        _is_partial: bool,
        frame: &Frame<'a>,
        context: Option<&'a Value<'a>>,
    ) -> Result<&'a Value<'a>> {
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

        let evaluated_args =
            Value::array_with_capacity(self.arena, args.len(), ArrayFlags::empty());

        if let Some(context) = context {
            evaluated_args.push(context);
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
        while let Value::Lambda {
            ref ast,
            input: lambda_input,
            frame: ref lambda_frame,
            ..
        } = result
        {
            if let AstKind::Lambda {
                ref body,
                thunk: true,
                ..
            } = ast.kind
            {
                if let AstKind::Function {
                    ref proc, ref args, ..
                } = body.kind
                {
                    let next = self.evaluate(proc, lambda_input, lambda_frame)?;
                    let evaluated_args =
                        Value::array_with_capacity(self.arena, args.len(), ArrayFlags::empty());

                    for arg in args {
                        let arg = self.evaluate(arg, lambda_input, lambda_frame)?;
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
        input: &'a Value<'a>,
        evaluated_proc: &'a Value<'a>,
        evaluated_args: &'a Value<'a>,
        frame: &Frame<'a>,
    ) -> Result<&'a Value<'a>> {
        match evaluated_proc {
            Value::Lambda {
                ref ast,
                ref frame,
                input,
                ..
            } => {
                if let AstKind::Lambda {
                    ref body, ref args, ..
                } = ast.kind
                {
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
                    self.evaluate(body, input, &frame)
                } else {
                    unreachable!()
                }
            }
            Value::NativeFn {
                ref name, ref func, ..
            } => {
                let context = self.fn_context(name, char_index, input, frame);
                func(context, evaluated_args)
            }
            _ => Err(Error::T1006InvokedNonFunction(char_index)),
        }
    }
}