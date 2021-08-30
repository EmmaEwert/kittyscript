use plex::lexer;

#[derive(Debug, Default, Clone, Copy)]
pub struct Span {
	pub lo: usize,
	pub hi: usize,
}

#[derive(Debug)]
pub enum Token {
	Semicolon,
	Whitespace,
	Equals,
	LeftParenthesis,
	RightParenthesis,
	Colon,
	LeftBrace,
	RightBrace,
	Comma,
	At,
	Float(f32),
	Integer(i32),
	Identifier(String),
	String(String),
}

lexer! {
	pub fn next_token(text: 'a) -> Token;
	r#";"#                      => Token::Semicolon,
	r#"[ \t\r\n]+"#             => Token::Whitespace,
	r#"="#                      => Token::Equals,
	r#"\("#                     => Token::LeftParenthesis,
	r#"\)"#                     => Token::RightParenthesis,
	r#":"#                      => Token::Colon,
	r#"{"#                      => Token::LeftBrace,
	r#"}"#                      => Token::RightBrace,
	r#","#                      => Token::Comma,
	r#"@"#                      => Token::At,
	r#"[0-9]+"#                 => Token::Integer(text.parse::<i32>().unwrap()),
	r#"[0-9]+\.[0-9]+"#         => Token::Float(text.parse::<f32>().unwrap()),
	r#"[^; \t\r\n=():{},"]+"#   => Token::Identifier(text.to_owned()),
	r#""[^"]+""#                => Token::String(text[1..text.len()-1].to_owned()),
	r#"."#                      => panic!("Unexpected character: {}", text),
}

pub struct Lexer<'a> {
	pub source: &'a str,
}

impl<'a> Iterator for Lexer<'a> {
	type Item = (Token, Span);
	fn next(&mut self) -> Option<(Token, Span)> {
		match next_token(self.source) {
			None => { None },
			Some((Token::Whitespace, source)) => { self.source = source; self.next() },
			Some((token, source)) => {
				println!("lexer: {:?}", token);
				self.source = source;
				Some((token, Span { lo: 0, hi: source.len() }))
			}
		}
	}
}
