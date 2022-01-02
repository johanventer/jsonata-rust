use std::collections::{hash_map, HashMap};

use super::ast::*;
use super::frame::Frame;
use super::json::Number;
use super::position::Position;
use super::value::{ArrayFlags, Value, ValueKind, ValuePool};
use super::{Error, Result};

pub struct Evaluator {
    pub pool: ValuePool,
}

impl Evaluator {
    pub fn new(pool: ValuePool) -> Self {
        Evaluator { pool }
    }

    #[inline]
    fn null(&self) -> Value {
        Value::new_null(self.pool.clone())
    }

    #[inline]
    fn bool(&self, b: bool) -> Value {
        Value::new_bool(self.pool.clone(), b)
    }

    #[inline]
    fn string(&self, s: &str) -> Value {
        Value::new_string(self.pool.clone(), s)
    }

    #[inline]
    fn number<T: Into<Number>>(&self, n: T) -> Value {
        Value::new_number(self.pool.clone(), n.into())
    }

    #[inline]
    pub fn array(&self, flags: ArrayFlags) -> Value {
        Value::new_array_with_flags(self.pool.clone(), flags)
    }

    #[inline]
    pub fn array_with_capacity(&self, capacity: usize, flags: ArrayFlags) -> Value {
        Value::new_array_with_capacity(self.pool.clone(), capacity, flags)
    }

    #[inline]
    fn object(&self) -> Value {
        Value::new_object(self.pool.clone())
    }

    pub fn evaluate(&self, node: &Node, input: Value, frame: Frame) -> Result<Value> {
        let mut result = match node.kind {
            NodeKind::Null => self.null(),
            NodeKind::Bool(b) => self.bool(b),
            NodeKind::String(ref s) => self.string(s),
            NodeKind::Number(n) => self.number(n),
            NodeKind::Block(ref exprs) => self.evaluate_block(exprs, input, frame.clone())?,
            NodeKind::Unary(ref op) => self.evaluate_unary_op(node, op, input, frame.clone())?,
            NodeKind::Binary(ref op, ref lhs, ref rhs) => {
                self.evaluate_binary_op(node, op, lhs, rhs, input, frame.clone())?
            }
            NodeKind::Var(ref name) => self.evaluate_var(name, input, frame.clone())?,
            NodeKind::Ternary {
                ref cond,
                ref truthy,
                ref falsy,
            } => self.evaluate_ternary(cond, truthy, falsy.as_deref(), input, frame.clone())?,
            NodeKind::Path(ref steps) => self.evaluate_path(node, steps, input, frame.clone())?,
            NodeKind::Name(ref name) => self.lookup(input, name),

            _ => unimplemented!("TODO: node kind not yet supported: {:#?}", node.kind),
        };

        if let Some(filters) = &node.predicates {
            for filter in filters {
                result = self.evaluate_filter(filter, result, frame.clone())?;
            }
        }

        Ok(if result.has_flags(ArrayFlags::SEQUENCE) {
            if node.keep_array {
                result.add_flags(ArrayFlags::SINGLETON);
            }
            if result.is_empty() {
                self.pool.undefined()
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

    fn evaluate_block(&self, exprs: &[Node], input: Value, frame: Frame) -> Result<Value> {
        let frame = Frame::new_with_parent(frame);
        if exprs.is_empty() {
            return Ok(self.pool.undefined());
        }

        let mut result = input;
        for expr in exprs {
            result = self.evaluate(expr, result.clone(), frame.clone())?;
        }

        Ok(result)
    }

    fn evaluate_var(&self, name: &str, input: Value, frame: Frame) -> Result<Value> {
        Ok(if name.is_empty() {
            if input.has_flags(ArrayFlags::WRAPPED) {
                input.get_member(0)
            } else {
                input
            }
        } else if let Some(value) = frame.lookup(name) {
            value
        } else {
            self.pool.undefined()
        })
    }

    fn evaluate_unary_op(
        &self,
        node: &Node,
        op: &UnaryOp,
        input: Value,
        frame: Frame,
    ) -> Result<Value> {
        match *op {
            UnaryOp::Minus(ref value) => {
                let result = self.evaluate(value, input, frame)?;
                let result = match *result.as_ref() {
                    ValueKind::Undefined => Ok(self.pool.undefined()),
                    ValueKind::Number(num) if !num.is_nan() => Ok(self.number(-num)),
                    _ => Err(Error::negating_non_numeric(node.position, result.clone())),
                };
                result
            }
            UnaryOp::ArrayConstructor(ref array) => {
                let mut result = self.array(if node.cons_array {
                    ArrayFlags::CONS
                } else {
                    ArrayFlags::empty()
                });
                for item in array.iter() {
                    let value = self.evaluate(item, input.clone(), frame.clone())?;
                    if let NodeKind::Unary(UnaryOp::ArrayConstructor(..)) = item.kind {
                        result.push_index(value.index);
                    } else {
                        result = self.append(result, value);
                    }
                }
                Ok(result)
            }
            UnaryOp::ObjectConstructor(ref object) => {
                self.evaluate_group_expression(node.position, object, input, frame)
            }
        }
    }

    fn evaluate_group_expression(
        &self,
        position: Position,
        object: &[(Node, Node)],
        input: Value,
        frame: Frame,
    ) -> Result<Value> {
        struct Group {
            pub data: Value,
            pub index: usize,
        }

        let mut groups: HashMap<String, Group> = HashMap::new();
        let input = input.wrap_in_array_if_needed(ArrayFlags::empty());

        if input.is_empty() {
            input.push(ValueKind::Undefined);
        }

        for item in input.members() {
            for (index, pair) in object.iter().enumerate() {
                let key = self.evaluate(&pair.0, item.clone(), frame.clone())?;
                if !key.is_string() {
                    return Err(Error::non_string_key(position, key));
                }

                match groups.entry(key.as_string()) {
                    hash_map::Entry::Occupied(mut entry) => {
                        let group = entry.get_mut();
                        if group.index != index {
                            return Err(Error::multiple_keys(position, key));
                        }
                        group.data = self.append(group.data.clone(), item.clone());
                    }
                    hash_map::Entry::Vacant(entry) => {
                        entry.insert(Group {
                            data: item.clone(),
                            index,
                        });
                    }
                };
            }
        }

        let result = self.object();

        for key in groups.keys() {
            let group = groups.get(key).unwrap();
            let value = self.evaluate(&object[group.index].1, group.data.clone(), frame.clone())?;
            if !value.is_undefined() {
                result.insert_index(key, value.index);
            }
        }

        Ok(result)
    }

    fn evaluate_binary_op(
        &self,
        node: &Node,
        op: &BinaryOp,
        lhs: &Node,
        rhs: &Node,
        input: Value,
        frame: Frame,
    ) -> Result<Value> {
        let rhs = self.evaluate(rhs, input.clone(), frame.clone())?;

        if *op == BinaryOp::Bind {
            if let NodeKind::Var(ref name) = lhs.kind {
                frame.bind(name, rhs);
            }
            return Ok(input);
        }

        let lhs = self.evaluate(lhs, input, frame)?;

        match op {
            BinaryOp::Add
            | BinaryOp::Subtract
            | BinaryOp::Multiply
            | BinaryOp::Divide
            | BinaryOp::Modulus => {
                let lhs = match *lhs.as_ref() {
                    ValueKind::Undefined => return Ok(self.pool.undefined()),
                    ValueKind::Number(n) if !n.is_nan() => f64::from(n),
                    _ => return Err(Error::left_side_not_number(node.position, op)),
                };

                let rhs = match *rhs.as_ref() {
                    ValueKind::Undefined => return Ok(self.pool.undefined()),
                    ValueKind::Number(n) if !n.is_nan() => f64::from(n),
                    _ => return Err(Error::right_side_not_number(node.position, op)),
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
                    Err(Error::NumberOfOutRange(result))
                } else {
                    Ok(self.number(result))
                }
            }

            BinaryOp::LessThan
            | BinaryOp::LessThanEqual
            | BinaryOp::GreaterThan
            | BinaryOp::GreaterThanEqual => {
                if lhs.is_undefined() || rhs.is_undefined() {
                    return Ok(self.pool.undefined());
                }

                if !((lhs.is_number() || lhs.is_string()) && (rhs.is_number() || rhs.is_string())) {
                    return Err(Error::binary_op_types(node.position, op));
                }

                if let (ValueKind::Number(ref lhs), ValueKind::Number(ref rhs)) =
                    (&*lhs.as_ref(), &*rhs.as_ref())
                {
                    let lhs = f64::from(*lhs);
                    let rhs = f64::from(*rhs);
                    return Ok(self.bool(match op {
                        BinaryOp::LessThan => lhs < rhs,
                        BinaryOp::LessThanEqual => lhs <= rhs,
                        BinaryOp::GreaterThan => lhs > rhs,
                        BinaryOp::GreaterThanEqual => lhs >= rhs,
                        _ => unreachable!(),
                    }));
                }

                if let (ValueKind::String(ref lhs), ValueKind::String(ref rhs)) =
                    (&*lhs.as_ref(), &*rhs.as_ref())
                {
                    return Ok(self.bool(match op {
                        BinaryOp::LessThan => lhs < rhs,
                        BinaryOp::LessThanEqual => lhs <= rhs,
                        BinaryOp::GreaterThan => lhs > rhs,
                        BinaryOp::GreaterThanEqual => lhs >= rhs,
                        _ => unreachable!(),
                    }));
                }

                Err(Error::binary_op_mismatch(node.position, lhs, rhs, op))
            }

            BinaryOp::Equal | BinaryOp::NotEqual => {
                if lhs.is_undefined() || rhs.is_undefined() {
                    return Ok(self.bool(false));
                }

                Ok(self.bool(match op {
                    BinaryOp::Equal => lhs == rhs,
                    BinaryOp::NotEqual => lhs != rhs,
                    _ => unreachable!(),
                }))
            }

            _ => unimplemented!("TODO: binary op not supported yet: {:#?}", *op),
        }
    }

    fn evaluate_ternary(
        &self,
        cond: &Node,
        truthy: &Node,
        falsy: Option<&Node>,
        input: Value,
        frame: Frame,
    ) -> Result<Value> {
        let cond = self.evaluate(cond, input.clone(), frame.clone())?;
        if self.boolean(cond) {
            self.evaluate(truthy, input, frame)
        } else if let Some(falsy) = falsy {
            self.evaluate(falsy, input, frame)
        } else {
            Ok(self.pool.undefined())
        }
    }

    fn evaluate_path(
        &self,
        node: &Node,
        steps: &[Node],
        input: Value,
        frame: Frame,
    ) -> Result<Value> {
        let mut input = if input.is_array() && !matches!(steps[0].kind, NodeKind::Var(..)) {
            input
        } else {
            input.wrap_in_array(ArrayFlags::empty())
        };

        let mut result = self.pool.undefined();

        for (index, step) in steps.iter().enumerate() {
            result = if index == 0 && step.cons_array {
                self.evaluate(step, input.clone(), frame.clone())?
            } else {
                self.evaluate_step(step, input.clone(), frame.clone(), index == steps.len() - 1)?
            };

            if result.is_undefined() || (result.is_array() && result.is_empty()) {
                break;
            }

            // if step.focus.is_none() {
            input = result.clone();
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

        if let Some((position, ref object)) = node.group_by {
            result = self.evaluate_group_expression(position, object, result, frame)?;
        }

        Ok(result)
    }

    fn evaluate_step(
        &self,
        step: &Node,
        input: Value,
        frame: Frame,
        last_step: bool,
    ) -> Result<Value> {
        let mut result = self.array(ArrayFlags::SEQUENCE);

        if let NodeKind::Sort(ref sorts) = step.kind {
            result = self.evaluate_sorts(sorts, input, frame.clone())?;
            if let Some(ref stages) = step.stages {
                result = self.evaluate_stages(stages, result, frame)?;
            }
            return Ok(result);
        }

        for item in input.members() {
            let mut item_result = self.evaluate(step, item, frame.clone())?;

            if let Some(ref stages) = step.stages {
                for stage in stages {
                    item_result = self.evaluate_filter(stage, item_result, frame.clone())?;
                }
            }

            if !item_result.is_undefined() {
                result.push_index(item_result.index);
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
                let result_sequence = self.array(ArrayFlags::SEQUENCE);

                for result_item in result.members() {
                    if !result_item.is_array() || result_item.has_flags(ArrayFlags::CONS) {
                        result_sequence.push_index(result_item.index);
                    } else {
                        for item in result_item.members() {
                            result_sequence.push_index(item.index);
                        }
                    }
                }
                result_sequence
            },
        )
    }

    fn evaluate_sorts(
        &self,
        _sorts: &[(Node, bool)],
        _inputt: Value,
        _frame: Frame,
    ) -> Result<Value> {
        unimplemented!("Sorts not yet implemented")
    }

    fn evaluate_stages(&self, _stages: &[Node], _input: Value, _frame: Frame) -> Result<Value> {
        unimplemented!("Stages not yet implemented")
    }

    fn evaluate_filter(&self, node: &Node, input: Value, frame: Frame) -> Result<Value> {
        let mut result = self.array(ArrayFlags::SEQUENCE);
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
            NodeKind::Filter(ref filter) => match filter.kind {
                NodeKind::Number(n) => {
                    let index = get_index(n);
                    let item = input.get_member(index as usize);
                    if !item.is_undefined() {
                        if item.is_array() {
                            result = item;
                        } else {
                            result.push_index(item.index);
                        }
                    }
                }
                _ => {
                    for (i, item) in input.members().enumerate() {
                        let mut index = self.evaluate(filter, item.clone(), frame.clone())?;
                        if index.is_number() && !index.is_nan() {
                            index = index.wrap_in_array(ArrayFlags::empty());
                        }
                        if index.is_array() && index.members().all(|v| v.is_number() && !v.is_nan())
                        {
                            index.members().for_each(|v| {
                                let index = get_index(v.as_f64());
                                if index == i {
                                    result.push_index(item.index);
                                }
                            });
                        } else if self.boolean(index) {
                            result.push_index(item.index);
                        }
                    }
                }
            },
            _ => unimplemented!("Filters other than numbers are not yet supported"),
        };

        Ok(result)
    }
}
