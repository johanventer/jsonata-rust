use super::value::{Value, ValuePool};
use super::Result;

pub struct Evaluator {
    pool: ValuePool,
}

impl Evaluator {
    pub fn new(pool: ValuePool) -> Self {
        Evaluator { pool }
    }

    pub fn evaluate(&self) -> Result<Value> {
        Ok(Value::new_undefined(self.pool.clone()))
    }
}
