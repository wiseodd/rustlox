use crate::callable::LoxCallable;

#[derive(strum_macros::Display, Clone, Debug)]
pub enum Object {
    String(String),
    Number(f64),
    Boolean(bool),
    Callable(LoxCallable),
    None,
}
