use crate::lexer::Span;

#[derive(Clone, Default)]
pub struct Expression {
	pub span: Span,
	pub node: Node,
}

impl std::fmt::Debug for Expression {
	fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
		print!("{:?}", self.node);
		Ok(())
	}
}

#[derive(Clone, Debug)]
pub enum Node {
	//Float(f32),
	Integer(i32),
	Identifier(String),
	String(String),
	Assignment(String, Box<Expression>),
	Call(String, Vec<Expression>),
	Empty,
	Partial(String, Box<Expression>),
	Function(Vec<Expression>, Vec<Expression>)
}

impl Default for Node {
	fn default() -> Self { Node::Empty }
}