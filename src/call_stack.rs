use crate::types::Expression;

#[derive(Debug, Clone, PartialEq)]
pub enum Control {
    Continue,
    Wait,
}

#[derive(Debug, Clone, PartialEq)]
struct Call {
    waiting: bool,
    pub expr: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ControlStack {
    call_stack: Vec<Call>,
    return_stack: Vec<Expression>,
}

impl ControlStack {
    pub fn new(first_expr: Expression) -> Self {
        ControlStack {
            call_stack: vec![Call {
                waiting: false,
                expr: first_expr,
            }],
            return_stack: vec![],
        }
    }
    pub fn size_call(&self) -> usize {
        self.call_stack.len()
    }

    pub fn push_return(&mut self, expr: Expression) {
        self.return_stack.push(expr);
    }

    pub fn pop_return(&mut self) -> Result<Expression, ()> {
        match self.return_stack.pop() {
            Some(expr) => Ok(expr),
            None => Err(()),
        }
    }

    pub fn pop_call(&mut self) -> Result<Expression, ()> {
        match self.call_stack.pop() {
            Some(call) => Ok(call.expr.clone()),
            None => Err(()),
        }
    }

    pub fn peek_call(&mut self) -> Result<Expression, ()> {
        match self.call_stack.last() {
            Some(call) => Ok(call.expr.clone()),
            None => Err(()),
        }
    }

    pub fn push_to_call_stack(&mut self, child_exprs: &[Expression]) -> Control {
        match self.call_stack.pop() {
            Some(
                call @ Call {
                    waiting: true,
                    expr: _,
                },
            ) => {
                self.call_stack.push(call);
                Control::Continue
            }
            Some(Call {
                waiting: false,
                expr,
            }) => {
                self.call_stack.push(Call {
                    waiting: true,
                    expr,
                });
                for child_expr in child_exprs {
                    self.call_stack.push(Call {
                        waiting: false,
                        expr: child_expr.clone(),
                    });
                }
                Control::Wait
            }
            None => Control::Continue,
        }
    }
}
