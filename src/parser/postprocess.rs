use super::ast::*;
use crate::error::*;
use crate::JsonAtaResult;

pub fn process_ast(node: &Node) -> JsonAtaResult<Node> {
    let mut result = match &node.kind {
        // Name nodes are wrapped in Path nodes
        NodeKind::Name(..) => Ok(Node::new_with_child(
            NodeKind::Path,
            node.position,
            node.clone(),
        )),
        NodeKind::Unary(ref op) => process_unary_node(node, op),
        NodeKind::Binary(ref op) => process_binary_node(node, op),
        NodeKind::Block => process_block_node(node),
        // TODO:
        //  - Function
        //  - Partial
        //  - Lambda
        //  - Ternary
        //  - Transform
        //  - Parent
        _ => Ok(node.clone()),
    }?;

    if node.keep_array {
        result.keep_array = true;
    }

    Ok(result)
}

fn process_children(node: &Node) -> JsonAtaResult<Node> {
    Ok(Node::new_with_children(
        node.kind.clone(),
        node.position,
        node.children
            .iter()
            .map(|child| process_ast(child))
            .collect::<JsonAtaResult<Vec<Node>>>()?,
    ))
}

fn process_unary_node(node: &Node, op: &UnaryOp) -> JsonAtaResult<Node> {
    Ok(match op {
        UnaryOp::Minus => process_unary_minus(node)?,
        UnaryOp::ArrayConstructor => process_children(node)?,
        UnaryOp::ObjectConstructor(object) => process_object_constructor(node, object)?,
    })
}

fn process_unary_minus(node: &Node) -> JsonAtaResult<Node> {
    let mut result = process_ast(&node.children[0])?;
    if let NodeKind::Num(ref mut num) = result.kind {
        *num = -*num;
    }
    Ok(result)
}

fn process_object_constructor(node: &Node, object: &Object) -> JsonAtaResult<Node> {
    let mut result_object: Object = Vec::with_capacity(object.len());
    for (k, v) in object.iter() {
        result_object.push((process_ast(k)?, process_ast(v)?));
    }
    Ok(Node::new(
        NodeKind::Unary(UnaryOp::ObjectConstructor(result_object)),
        node.position,
    ))
}

fn process_binary_node(node: &Node, op: &BinaryOp) -> JsonAtaResult<Node> {
    match op {
        BinaryOp::PathOp => process_path(node),
        BinaryOp::Predicate => process_predicate(node),
        BinaryOp::GroupBy(object) => process_group_by(node, object),
        BinaryOp::SortOp => process_sort(node),
        BinaryOp::ContextBind => process_context_bind(node),
        BinaryOp::PositionalBind => process_positional_bind(node),
        _ => process_children(node),
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
        if rhs.predicate.is_some() {
            rhs.stages = Some(vec![*rhs.predicate.unwrap()]);
            rhs.predicate = None;
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

fn process_predicate(node: &Node) -> JsonAtaResult<Node> {
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
        step.predicate = Some(box predicate);
    }

    Ok(result)
}

fn process_group_by(node: &Node, object: &Object) -> JsonAtaResult<Node> {
    let mut result = process_ast(&node.children[0])?;

    if result.group_by.is_some() {
        return Err(box S0210 {
            position: node.position,
        });
    }

    let mut result_object: Object = Vec::with_capacity(object.len());
    for (k, v) in object.iter() {
        result_object.push((process_ast(k)?, process_ast(v)?));
    }

    result.group_by = Some(result_object);

    Ok(result)
}

fn process_sort(node: &Node) -> JsonAtaResult<Node> {
    let mut result = process_ast(&node.children[0])?;

    if !result.is_path() {
        result = Node::new_with_child(NodeKind::Path, node.position, result);
    }

    let mut sort_terms: Vec<Node> = vec![];
    for sort_term in &node.children[1..node.children.len() - 1] {
        if let NodeKind::SortTerm(desc) = sort_term.kind {
            let expr = process_ast(&sort_term.children[0])?;
            sort_terms.push(Node::new_with_child(
                NodeKind::SortTerm(desc),
                sort_term.position,
                expr,
            ))
        } else {
            unreachable!("Node should've been a SortTerm")
        }
    }

    let sort = Node::new_with_children(NodeKind::Sort, node.position, sort_terms);

    result.children.push(sort);

    Ok(result)
}

fn process_context_bind(node: &Node) -> JsonAtaResult<Node> {
    // TODO
    // unimplemented!("Context bind not yet supported")
    Ok(node.clone())
}

fn process_positional_bind(node: &Node) -> JsonAtaResult<Node> {
    // TODO
    // unimplemented!("Positional bind not yet supported")
    Ok(node.clone())
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
