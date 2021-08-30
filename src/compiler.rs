use inkwell::values::CallableValue;
use std::collections::HashMap;
use std::convert::TryFrom;
use inkwell::AddressSpace;
use inkwell::context::Context;
use inkwell::builder::Builder;
use inkwell::module::Module;
use inkwell::passes::PassManager;
use inkwell::targets::TargetTriple;
use inkwell::types::BasicTypeEnum;
use inkwell::values::FunctionValue;
use inkwell::values::PointerValue;
use crate::ast::*;

struct Compiler<'a, 'context> {
	context: &'context Context,
	module: &'a Module<'context>,
	builder: &'a Builder<'context>,
	pass_manager: &'a PassManager<FunctionValue<'context>>,
	ast: &'a Vec<Expression>,

	variables: HashMap<String, PointerValue<'context>>,
	functions: HashMap<String, FunctionValue<'context>>,
}

impl<'a, 'context> Compiler<'a, 'context> {
	fn compile(&mut self) -> Result<(), String> {
		self.compile_intrinsics();
		let i32_type = self.context.i32_type();
		let fn_type = i32_type.fn_type(&[], false);
		let fn_value = self.module.add_function("main", fn_type, None);
		let main_block = self.context.append_basic_block(fn_value, "main");
		self.builder.position_at_end(main_block);
		for expression in self.ast {
			match self.compile_expression(expression) {
				Err(message) => return Err(message),
				Ok(_) => { }
			}
		}
		let zero = i32_type.const_int(0, false);
		self.builder.build_return(Some(&zero));
		self.pass_manager.run_on(&fn_value);
		Ok(())
	}

	fn compile_expression(&mut self, expression : &Expression) -> Result<PointerValue<'context>, String> {
		match &expression.node {
			Node::Call(name, expressions)      => self.compile_call(name.to_string(), expressions.to_vec()),
			Node::Assignment(name, expression) => self.compile_assignment(name.to_string(), expression),
			Node::Integer(value)               => self.compile_integer(*value),
			Node::Identifier(name)             => self.compile_identifier(name.to_string()),
			Node::String(string)               => self.compile_string(string.replace(r"\n", "\n")),
			Node::Function(_, _)               => Err(format!("unassigned function: {:?}", expression.node)),
			_                                  => Err(format!("not yet implemented: {:?}", expression.node))
		}
	}

	fn compile_assignment(&mut self, name : String, expression : &Expression) -> Result<PointerValue<'context>, String> {
		match &expression.node {
			Node::Function(arguments, expressions) => self.compile_function(name, arguments.to_vec(), expressions.to_vec()),
			_ => {
				match self.compile_expression(expression) {
					Ok(value) => {
						let alloca = if self.variables.contains_key(&name) {
							self.variables[&name]
						} else {
							let i32_type = self.context.i32_type();
							let i32_ptr_type = i32_type.ptr_type(AddressSpace::Generic);
							self.builder.build_alloca(i32_ptr_type, &name)
						};
						self.builder.build_store(alloca, value);
						self.variables.insert(name, alloca);
						Ok(value)
					},
					Err(message) => Err(message)
				}
			}
		}
	}

	fn compile_call(&mut self, name : String, expressions: Vec<Expression>) -> Result<PointerValue<'context>, String> {
		let callable_value = match self.functions.get(&name) {
			Some(fn_value) => CallableValue::from(*fn_value),
			None => CallableValue::try_from(*self.variables.get(&name).unwrap()).unwrap()
		};
		let arguments = expressions
			.iter()
			.filter_map(|expression| self.compile_expression(&expression).ok())
			.collect::<Vec<_>>()
			.iter()
			.map(|argument| self.builder.build_load(*argument, "arg"))
			.collect::<Vec<_>>();
		let call = self.builder.build_call(callable_value, &arguments, &name)
			.try_as_basic_value()
			.left()
			.unwrap()
			.into_int_value();
		let alloca = self.builder.build_alloca(self.context.i32_type(), &name);
		self.builder.build_store(alloca, call);
		Ok(alloca)
	}

	fn compile_function(&mut self, name: String, arguments: Vec<Expression>, expressions: Vec<Expression>) -> Result<PointerValue<'context>, String> {
		let i32_type = self.context.i32_type();
		let old_block = self.builder.get_insert_block();
		let mut argument_types = vec![];
		for _argument in &arguments {
			argument_types.push(BasicTypeEnum::PointerType(i32_type.ptr_type(AddressSpace::Generic)))
		}
		let fn_type = i32_type.ptr_type(AddressSpace::Generic).fn_type(&argument_types, false);
		let fn_value = self.module.add_function(&name, fn_type, None);
		self.functions.insert(name, fn_value);
		for (index, argument) in fn_value.get_params().into_iter().enumerate() {
			match &arguments[index].node {
				Node::Identifier(name) => {
					self.variables.insert(name.to_string(), argument.into_pointer_value());
				}
				_ => return Err(format!("unknown type for argument {}", index))
			}
		}
		let fn_block = self.context.append_basic_block(fn_value, "fn");
		self.builder.position_at_end(fn_block);

		let mut tail : Result<PointerValue, String> = Err("no tail".to_string());
		for expression in expressions {
			tail = self.compile_expression(&expression)
		}
		match tail {
			Ok(value) => {
				self.builder.build_return(Some(&value));
				self.builder.position_at_end(old_block.unwrap());
				Ok(value)
			}
			error => error
		}
	}

	fn compile_integer(&mut self, value: i32) -> Result<PointerValue<'context>, String> {
		let i32_type = self.context.i32_type();
		let int = i32_type.const_int(value as u64, false);
		let alloca = self.builder.build_alloca(i32_type, "int");
		self.builder.build_store(alloca, int);
		Ok(alloca)
	}

	fn compile_identifier(&mut self, name: String) -> Result<PointerValue<'context>, String> {
		match self.variables.get(&name) {
			None => Err(format!("no variable {}", name)),
			Some(pointer) => Ok(*pointer)
		}
	}

	fn compile_string(&mut self, string: String) -> Result<PointerValue<'context>, String> {
		Ok(self.builder.build_global_string_ptr(&string, "str").as_pointer_value())
	}

	fn compile_intrinsics(&mut self) {
		let i8_type = self.context.i8_type();
		let i8_ptr_type = i8_type.ptr_type(AddressSpace::Generic);
		let i32_type = self.context.i32_type();
		let i32_ptr_type = i32_type.ptr_type(AddressSpace::Generic);

		// extern printf
		let printf_type = i32_type.fn_type(&[i8_ptr_type.into()], true);
		let printf_value = self.module.add_function("printf", printf_type, None);
		// self.functions.insert("printf".to_string(), printf_value);

		let print_type = i32_type.fn_type(&[i32_ptr_type.into()], false);
		let print_value = self.module.add_function("print", print_type, None);
		self.functions.insert("print".to_string(), print_value);
		let print_block = self.context.append_basic_block(print_value, "print");
		self.builder.position_at_end(print_block);
		let arg = print_value.get_first_param().unwrap();
		let string = self.builder.build_global_string_ptr("%d\n", "str").as_pointer_value();
		let load = self.builder.build_load(arg.into_pointer_value(), "load");
		let call = self.builder.build_call(printf_value, &[string.into(), load], "printf");
		self.builder.build_return(Some(&call.try_as_basic_value().left().unwrap()));

		// +
		let add_type = i32_type.fn_type(&[i32_ptr_type.into(), i32_ptr_type.into()], false);
		let add_value = self.module.add_function("+", add_type, None);
		self.functions.insert("+".to_string(), add_value);
		let add_basic_block = self.context.append_basic_block(add_value, "+");
		let arg0 = add_value.get_first_param().unwrap().into_pointer_value();
		let arg1 = add_value.get_last_param().unwrap().into_pointer_value();
		self.builder.position_at_end(add_basic_block);
		let a = self.builder.build_load(arg0, "a");
		let b = self.builder.build_load(arg1, "b");
		let add = self.builder.build_int_add(a.into_int_value(), b.into_int_value(), "add");
		self.builder.build_return(Some(&add));
	}
}

pub fn compile(ast : Vec<Expression>) -> Result<String, String> {
	let context = Context::create();
	let module = context.create_module("module");
	let builder = context.create_builder();
	let pass_manager = PassManager::create(&module);

	module.set_triple(&TargetTriple::create("x86_64-pc-linux-gnu"));

	//pass_manager.add_promote_memory_to_register_pass();
	pass_manager.initialize();

	let mut compiler = Compiler {
		context: &context,
		module: &module,
		builder: &builder,
		pass_manager: &pass_manager,
		ast: &ast,
		variables: HashMap::new(),
		functions: HashMap::new(),
	};

	match compiler.compile() {
		Ok(_) => {
			match compiler.module.print_to_file("main.ll") {
				Err(message) => eprintln!("compiler: {}", message),
				_ => { },
			}
			Ok(compiler.module.print_to_string().to_string())
		}
		Err(message) => Err(format!("{}\n{}", message, compiler.module.print_to_string().to_string())),
	}

}
