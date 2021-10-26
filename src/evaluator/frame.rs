// use std::cell::RefCell;
// use std::collections::HashMap;
// use std::rc::Rc;

// use super::value::Value;

// pub type FramePtr = Rc<RefCell<Frame>>;

// #[derive(Debug)]
// pub struct Frame {
//     bindings: HashMap<String, Rc<Value>>,
//     parent_frame: Option<FramePtr>,
// }

// impl Frame {
//     pub(crate) fn new() -> Self {
//         Self {
//             bindings: HashMap::new(),
//             parent_frame: None,
//         }
//     }

//     /// Creates a new empty frame, with a parent frame for lookups
//     pub(crate) fn with_parent(parent_frame: FramePtr) -> Self {
//         Self {
//             bindings: HashMap::new(),
//             parent_frame: Some(parent_frame),
//         }
//     }

//     pub(crate) fn new_ptr() -> FramePtr {
//         Rc::new(RefCell::new(Frame::new()))
//     }

//     pub(crate) fn ptr_with_parent(parent_frame: FramePtr) -> FramePtr {
//         Rc::new(RefCell::new(Frame::with_parent(parent_frame)))
//     }

//     /// Bind a value to a name in a frame
//     pub(crate) fn bind(&mut self, name: &str, value: Rc<Value>) {
//         &self.bindings.insert(name.to_string(), value);
//     }

//     /// Lookup a value by name in a frame
//     pub(crate) fn lookup(&self, name: &str) -> Option<Rc<Value>> {
//         match self.bindings.get(name) {
//             Some(value) => Some(Rc::clone(value)),
//             None => match &self.parent_frame {
//                 Some(parent) => parent.borrow().lookup(name),
//                 None => None,
//             },
//         }
//     }
// }
