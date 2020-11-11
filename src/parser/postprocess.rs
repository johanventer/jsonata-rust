use super::ast::*;
use crate::error::*;
use crate::JsonAtaResult;

pub fn process_ast(node: &Node) -> JsonAtaResult<Node> {
    let mut result = match &node.kind {
        // Name nodes are wrapped in Path nodes
        NodeKind::Name(..) => Ok(Node::new_with_child(NodeKind::Path, node.position, node.clone())),
        NodeKind::Unary(ref op) => process_unary_node(node, op),
        NodeKind::Binary(ref op) => process_binary_node(node, op),
        NodeKind::Block => process_block_node(node),
        _ => Ok(node.clone()),
    }?;

    if node.keep_array {
        result.keep_array = true;
    }

    Ok(result)
}

fn process_unary_node(node: &Node, op: &UnaryOp) -> JsonAtaResult<Node> {
    Ok(match op {
        UnaryOp::Minus => process_unary_minus(node)?,
        UnaryOp::ArrayConstructor => process_array_constructor(node)?,
        UnaryOp::ObjectConstructor => process_object_constructor(node)?,
    })
}

fn process_unary_minus(node: &Node) -> JsonAtaResult<Node> {
    let mut result = process_ast(&node.children[0])?;
    if let NodeKind::Num(ref mut num) = result.kind {
        *num = -*num;
    }
    Ok(result)
}

fn process_array_constructor(node: &Node) -> JsonAtaResult<Node> {
    // TODO
    Ok(node.clone())
}

fn process_object_constructor(node: &Node) -> JsonAtaResult<Node> {
    // TODO
    Ok(node.clone())
}

fn process_binary_node(node: &Node, op: &BinaryOp) -> JsonAtaResult<Node> {
    match op {
        BinaryOp::PathOp => process_path(node),
        BinaryOp::ArrayPredicate => process_array_predicate(node),
        BinaryOp::GroupBy => process_group_by(node),
        _ => {
            let lhs = process_ast(&node.children[0])?;
            let rhs = process_ast(&node.children[1])?;
            Ok(Node::new_with_children(
                node.kind.clone(),
                node.position,
                vec![lhs, rhs],
            ))
        }
    }
}

fn process_path(node: &Node) -> JsonAtaResult<Node> {
    let lhs = process_ast(&node.children[0])?;
    let mut rhs = process_ast(&node.children[1])?;

    let mut result = {
        // If lhs is a Path, start with that, otherwise create a new one
        if lhs.is_path() {
            lhs
        } else {
            Node::new_with_child(NodeKind::Path, lhs.position, lhs)
        }
    };

    // TODO: If the lhs is a Parent (parser.js:997)

    // TODO: If the rhs is a Function (parser.js:1001)

    // If rhs is a Path, merge the steps in
    if rhs.is_path() {
        result.children.append(&mut rhs.children);
    } else {
        if rhs.predicates.is_some() {
            rhs.stages = rhs.predicates;
            rhs.predicates = None;
        }
        result.children.push(rhs);
    }

    let last_index = result.children.len() - 1;
    let mut keep_array = false;

    for (step_index, step) in result.children.iter_mut().enumerate() {
        match step.kind {
            // Steps cannot be literal values
            NodeKind::Num(..) | NodeKind::Bool(..) | NodeKind::Null => {
                return Err(box S0213 {
                    position: step.position,
                    value: step.kind.to_string(),
                })
            }
            // Steps that are string literals should be switched to Name
            NodeKind::Str(ref v) => {
                step.kind = NodeKind::Name(v.clone());
            }
            // If first or last step is an array constructor, it shouldn't be flattened
            NodeKind::Unary(ref op) => {
                if let UnaryOp::ArrayConstructor = op {
                    if step_index == 0 || step_index == last_index {
                        step.keep_array = true;
                    }
                }
            }
            _ => (),
        }

        keep_array = keep_array || step.keep_array;
    }

    result.keep_array = keep_array;

    Ok(result)
}

fn process_array_predicate(node: &Node) -> JsonAtaResult<Node> {
    println!("PROCESSING: {:#?}", node.children[0]);

    let mut result = process_ast(&node.children[0])?;
    let mut is_stages = false;

    let step = if result.is_path() {
        is_stages = true;
        let last_index = result.children.len() - 1;
        &mut result.children[last_index]
    } else {
        &mut result
    };

    if step.group_by.is_some() {
        return Err(box S0209 {
            position: node.position,
        });
    }

    let predicate = process_ast(&node.children[1])?;

    // TODO: seekingParent (parser.js:1074)

    if is_stages {
        if step.stages.is_none() {
            step.stages = Some(vec![predicate]);
        } else {
            if let Some(ref mut stages) = step.stages {
                stages.push(predicate);
            }
        }
    } else {
        if step.predicates.is_none() {
            step.predicates = Some(vec![predicate]);
        } else {
            if let Some(ref mut predicates) = step.predicates {
                predicates.push(predicate);
            }
        }
    }

    Ok(result)
}

fn process_group_by(node: &Node) -> JsonAtaResult<Node> {
    let mut result = process_ast(&node.children[0])?;

    if result.group_by.is_some() {
        return Err(box S0210 {
            position: node.position,
        });
    }

    let mut object: Object = vec![];

    for i in 1..node.children.len() - 2 {
        object.push((
            process_ast(&node.children[i])?,
            process_ast(&node.children[i + 1])?,
        ));
    }

    result.group_by = Some(GroupBy {
        position: node.position,
        object,
    });

    Ok(result)
}

fn process_block_node(node: &Node) -> JsonAtaResult<Node> {
    let children = node
        .children
        .iter()
        .map(|child| {
            process_ast(child)

            // TODO: consarray (parser.js:1267)
        })
        .collect::<JsonAtaResult<Vec<Node>>>()?;

    Ok(Node::new_with_children(
        NodeKind::Block,
        node.position,
        children,
    ))
}
// fn process_ast(&mut self, node: Node) -> Node {
//     use NodeKind::*;
//     let mut node = node;

//     match &node.kind {
//         Parent(..) => {
//             node.kind = Parent(Some(Slot {
//                 label: format!("!{}", self.ancestor_label),
//                 level: 1,
//                 index: self.ancestor_index,
//             }));

//             self.ancestor_index += 1;
//             self.ancestor_label += 1;
//             node
//         }
//         // Wrap Name nodes in a Path node
//         Name(name) => {
//             let keep_array = node.keep_array;
//             let mut path = Node::new_with_child(Path, node.position, node);
//             path.keep_array = keep_array;
//             path
//             // TODO: seeking_parent
//         }
//         Unary(ref op) => match op {
//             // Array constructor - process each child
//             UnaryOp::Array => {
//                 // TODO: let consarray = node.consarray;
//                 node.children = node
//                     .children
//                     .into_iter()
//                     .map(|child| self.process_ast(child))
//                     .collect();
//                 node
//             }
//             UnaryOp::Minus => {
//                 let expression = &mut node.children[0];
//                 // Pre-process unary minus on numbers
//                 if let Num(ref mut num) = expression.kind {
//                     *num = -*num;
//                 } else {
//                     // pushAncestry
//                 }
//                 node
//             }
//         },
//         Transform | Object => {
//             node.children = node
//                 .children
//                 .into_iter()
//                 .map(|child| self.process_ast(child))
//                 .collect();
//             node
//         }
//         _ => node,
//     }

//    macro_rules! binary {
//        ($t:tt, $n:ident) => {{
//            let lhs = self.process_ast($n.lhs);
//            let rhs = self.process_ast($n.rhs);
//            // pushAncestory for both lhs and rhs
//            Box::new($t(BinaryNode {
//                position: $n.position,
//                lhs,
//                rhs,
//            }))
//        }};
//    }

//    /* Things to cover here:
//        [x] PathSeparator
//        [x] Name -> Gets wrapped in a path
//        [x] Chain -> Returns an Apply node
//        [x] ParentOp
//        [x] FunctionCall
//        [x] PartialFunctionCall
//        [x] LambdaFunction
//        [x] UnaryMinus
//        [x] Block
//        [x] Array
//        [x] Assignment -> Returns a Bind node
//        [x] OrderBy
//        [x] Ternary
//        [x] Transform
//        [x] Object
//        [ ] GroupBy
//        [ ] ArrayPredicate
//        [ ] FocusVariableBind
//        [ ] IndexVariableBind
//    */
//    match *ast {
//        PathSeparator(node) => {
//            let mut result: Box<Node>;
//            let lhs = self.process_ast(node.lhs);

//            if let Path(_) = *lhs {
//                // Left hand side is a Path, so let's start with that
//                result = lhs;
//            } else if let Parent(node) = *lhs {
//                // Let hand side is a parent, so we will be looking for a parent
//                result = Box::new(Path(PathNode {
//                    steps: vec![],
//                    seeking_parent: vec![node.slot],
//                    keep_singleton_array: false,
//                }));
//            } else {
//                // Otherwise we are creating a new path, where the left hand side will be the
//                // first step
//                result = Box::new(Path(PathNode {
//                    steps: vec![lhs],
//                    seeking_parent: vec![],
//                    keep_singleton_array: false,
//                }));
//            }

//            let mut rhs = self.process_ast(node.rhs);
//            /*
//             TODO: This needs implementing
//                        if (rest.type === 'function' &&
//                            rest.procedure.type === 'path' &&
//                            rest.procedure.steps.length === 1 &&
//                            rest.procedure.steps[0].type === 'name' &&
//                            result.steps[result.steps.length - 1].type === 'function') {
//                            // next function in chain of functions - will override a thenable
//                            result.steps[result.steps.length - 1].nextFunction = rest.procedure.steps[0].value;
//                        }
//            */
//            if let Path(result) = result.as_mut() {
//                if let Path(node) = rhs.as_mut() {
//                    // Right hand side is a path, so it must be merged with our result
//                    result.steps.append(&mut node.steps);
//                } else {
//                    /*
//                    TODO: Figure out what predicate and stages are valid for
//                    if(typeof rest.predicate !== 'undefined') {
//                        rest.stages = rest.predicate;
//                        delete rest.predicate;
//                    }
//                    */
//                    result.steps.push(rhs);
//                }

//                for step in &mut result.steps {
//                    let mut replace = false;
//                    match step.as_ref() {
//                        // Don't allow steps to be numbers, null, or boolean values
//                        Number(node) => error!(s0213, node.get_position(), &node.get_value()),
//                        Null(node) => error!(s0213, node.get_position(), &node.get_value()),
//                        Boolean(node) => error!(s0213, node.get_position(), &node.get_value()),

//                        // Any steps within a path that are string literals should be changed to names
//                        Str(node) => replace = true,

//                        _ => (),
//                    }
//                    if replace {
//                        *step = Box::new(Name(LiteralNode::new(
//                            step.get_position(),
//                            step.get_value(),
//                        )));
//                    }
//                }
//                // Any step that signal keeping a singleton array, should be flagged on the path
//                if result.steps.iter().any(|step| match step.as_ref() {
//                    Name(node) => node.keep_array,
//                    _ => false,
//                }) {
//                    result.keep_singleton_array = true;
//                }

//                // If first step is a path constructor, flag it for special handling
//                if let Some(Array(node)) = result.steps.first_mut().map(|b| b.as_mut()) {
//                    node.consarray = true;
//                }
//                // If last step is a path constructor, flag it for special handling
//                if let Some(Array(node)) = result.steps.last_mut().map(|b| b.as_mut()) {
//                    node.consarray = true;
//                }

//                // self.resolve_ancestry(result);
//            }

//            result
//        }
//        // Block (array of expressions) - process each node
//        Block(node) => {
//            let mut expressions = Vec::new();
//            let mut consarray = false;
//            let position = node.get_position();
//            for expr in node.expressions {
//                let expr = self.process_ast(expr);
//                match *expr {
//                    Array(ref node) => {
//                        if node.consarray {
//                            consarray = true;
//                        }
//                    }
//                    Path(ref node) => {
//                        if !node.steps.is_empty() {
//                            if let Array(ref node) = *node.steps[0] {
//                                if node.consarray {
//                                    consarray = true
//                                }
//                            }
//                        }
//                    }
//                    _ => (),
//                }
//                // pushAncestry(result, value)
//                expressions.push(expr);
//            }
//            Box::new(Block(ExpressionsNode {
//                position,
//                expressions,
//                consarray,
//            }))
//        }
//        // Ternary conditional
//        Ternary(node) => {
//            let position = node.get_position();
//            let condition = self.process_ast(node.condition);
//            // pushAncestry(result, result.condition)
//            let then = self.process_ast(node.then);
//            // pushAncestry(result, result.then)
//            let els = match node.els {
//                Some(node) => {
//                    let node = self.process_ast(node);
//                    // pushAncestry(result, node)
//                    Some(node)
//                }
//                None => None,
//            };
//            Box::new(Ternary(TernaryNode {
//                position,
//                condition,
//                then,
//                els,
//            }))
//        }
//        // Assignment
//        Assignment(node) => {
//            let lhs = self.process_ast(node.lhs);
//            let rhs = self.process_ast(node.rhs);
//            // pushAncestry(result, result.rhs)
//            Box::new(Bind(BindNode {
//                position: node.position,
//                lhs,
//                rhs,
//            }))
//        }
//        // Function application
//        Chain(node) => {
//            let lhs = self.process_ast(node.lhs);
//            let rhs = self.process_ast(node.rhs);
//            // pushAncestry(result, result.rhs)
//            Box::new(Apply(ApplyNode {
//                position: node.position,
//                lhs,
//                rhs,
//            }))
//        }
//        FunctionCall(node) => {
//            let mut arguments = Vec::new();
//            for arg in node.arguments {
//                let arg = self.process_ast(arg);
//                // pushAncestory
//                arguments.push(arg);
//            }
//            let procedure = self.process_ast(node.procedure);
//            Box::new(FunctionCall(FunctionCallNode {
//                position: node.position,
//                arguments,
//                procedure,
//            }))
//        }
//        PartialFunctionCall(node) => {
//            let mut arguments = Vec::new();
//            for arg in node.arguments {
//                let arg = self.process_ast(arg);
//                // pushAncestory
//                arguments.push(arg);
//            }
//            let procedure = self.process_ast(node.procedure);
//            Box::new(PartialFunctionCall(FunctionCallNode {
//                position: node.position,
//                arguments,
//                procedure,
//            }))
//        }
//        LambdaFunction(node) => {
//            let body = self.process_ast(node.body);
//            Box::new(LambdaFunction(LambdaNode {
//                position: node.position,
//                arguments: node.arguments,
//                body,
//            }))
//            // TODO: Tail call optimization
//        }
//        // Order by
//        //  LHS is the array to be ordered
//        //  RHS defines the terms
//        OrderBy(node) => {
//            let mut lhs = self.process_ast(node.lhs);
//            let mut terms = Vec::new();

//            for term in node.rhs {
//                let expression = self.process_ast(term.expression);
//                // pushAncestory
//                terms.push(SortTermNode {
//                    position: term.position,
//                    descending: term.descending,
//                    expression,
//                })
//            }

//            let sort = Box::new(Sort(SortNode {
//                position: node.position,
//                terms,
//            }));

//            if let Path(ref mut node) = lhs.as_mut() {
//                node.steps.push(sort);
//                lhs
//            } else {
//                Box::new(Path(PathNode {
//                    steps: vec![sort],
//                    seeking_parent: vec![],
//                    keep_singleton_array: false,
//                }))
//            }
//        }
//        // // Positional variable binding
//        // IndexVariableBind(node) => {

//        // },
//        // // Context variable binding
//        // FocusVariableBind(node) => {

//        // }
//        // Group by
//        //  LHS is a step or a predicated step
//        //  RHS is the object constructor expression
//        // GroupBy(node) => {
//        //     let mut result = self.process_ast(node.lhs);
//        //     result
//        // }
//        // Predicated step:
//        //  LHS is a step or a predicated step
//        //  RHS is the predicate expression
//        //ArrayPredicate(node) => {
//        //    let mut result = self.process_ast(node.lhs);
//        //    let mut step = &result;
//        //    let mut is_stages = false;

//        //    if let Path(node) = *result {
//        //        if node.steps.len() > 0 {
//        //            is_stages = true;
//        //            step = node.steps.last().unwrap();
//        //        }
//        //    }
//        //    //                         if (typeof step.group !== 'undefined') {
//        //    //                             throw {
//        //    //                                 code: "S0209",
//        //    //                                 stack: (new Error()).stack,
//        //    //                                 position: expr.position
//        //    //                             };
//        //    //                         }
//        //    //
//        //    //

//        //    let predicate = self.process_ast(node.rhs);

//        //    // /*
//        //    //                         var predicate = processAST(expr.rhs);
//        //    //                         if(typeof predicate.seekingParent !== 'undefined') {
//        //    //                             predicate.seekingParent.forEach(slot => {
//        //    //                                 if(slot.level === 1) {
//        //    //                                     seekParent(step, slot);
//        //    //                                 } else {
//        //    //                                     slot.level--;
//        //    //                                 }
//        //    //                             });
//        //    //                             pushAncestry(step, predicate);
//        //    //                         }
//        //    //                         step[type].push({type: 'filter', expr: predicate, position: expr.position});
//        //    //                         break;
//        //    // // */
//        //}
//        Add(node) => binary!(Add, node),
//        Subtract(node) => binary!(Subtract, node),
//        Multiply(node) => binary!(Multiply, node),
//        Divide(node) => binary!(Divide, node),
//        Modulus(node) => binary!(Modulus, node),
//        Equal(node) => binary!(Equal, node),
//        LessThan(node) => binary!(LessThan, node),
//        GreaterThan(node) => binary!(GreaterThan, node),
//        NotEqual(node) => binary!(NotEqual, node),
//        LessThanEqual(node) => binary!(LessThanEqual, node),
//        GreaterThanEqual(node) => binary!(GreaterThanEqual, node),
//        Concat(node) => binary!(Concat, node),
//        And(node) => binary!(And, node),
//        Or(node) => binary!(Or, node),
//        In(node) => binary!(In, node),
//        Range(node) => binary!(Range, node),
// _ => node,
// }
// }
