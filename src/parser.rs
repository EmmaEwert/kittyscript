use plex::parser;
use crate::lexer::*;
use crate::lexer::Token::*;
use crate::ast::*;

parser! {
	fn parse_(Token, Span);

	(a, b) {
		Span {
			lo: a.lo,
			hi: b.hi,
		}
	}

	expressions: Vec<Expression> {
		=> vec![],
		expression[expression] => { vec![expression] },
		expressions[mut expressions] Semicolon expression[expression] => {
			expressions.push(expression);
			expressions
		},
	}

	expression: Expression {
		Identifier(name) Equals expression[expression] => Expression {
			span: span!(),
			node: Node::Assignment(name, Box::new(expression)),
		},
		atom[atom] expression0[operator] => Expression {
			span: span!(),
			node: match operator.node {
				Node::Empty => { atom.node },
				Node::Partial(name, right) => {
					Node::Call(name, vec![
						Expression {
							span: span!(),
							node: atom.node
						},
						*right
					])
				}
				_ => panic!("at the disco")
			}
		},
		LeftParenthesis parameters[parameters] LeftBrace expressions[expressions] RightBrace => Expression {
			span: span!(),
			node: Node::Function(parameters, expressions),
		},
		Identifier(name) LeftParenthesis /*expression[expression]*/ arguments[arguments] => Expression {
			span: span!(),
			node: Node::Call(name, arguments)
		},
	}

	expression0: Expression {
		=> Expression {
			span: Span { lo: 0, hi: 0 },
			node: Node::Empty
		},
		Identifier(name) expression[expression] => Expression {
			span: span!(),
			node: Node::Partial(name, Box::new(expression))
		},
	}

	atom: Expression {
		Identifier(name) => Expression { span: span!(), node: Node::Identifier(name) },
		Integer(value) => Expression { span: span!(), node: Node::Integer(value) },
		String(string) => Expression { span: span!(), node: Node::String(string) },
	}

	arguments: Vec<Expression> {
		RightParenthesis => vec![],
		expression[argument] RightParenthesis => vec![argument],
		expression[argument] Comma arguments[mut arguments] => {
			arguments.insert(0, argument);
			arguments
		},
	}

	parameters: Vec<Expression> {
		RightParenthesis => vec![],
		Identifier(name) RightParenthesis => vec![
			Expression { span: span!(), node: Node::Identifier(name) }
		],
		Identifier(name) Comma parameters[mut parameters] => {
			parameters.insert(0, Expression { span: span!(), node: Node::Identifier(name) });
			parameters
		}
	}
}

pub fn parse<I: Iterator<Item = (Token, Span)>>(i : I) -> Result<Vec<Expression>, (Option<(Token, Span)>, &'static str)> {
	parse_(i)
}
