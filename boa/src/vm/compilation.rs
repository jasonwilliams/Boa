use super::*;
use crate::{
    exec::Executable, syntax::ast::Const, syntax::ast::Node, value::RcBigInt, value::RcString,
};

#[derive(Debug, Default)]
pub struct Compiler {
    pub(super) instructions: Vec<Instruction>,
    pub(super) pool: Vec<Value>,
}

impl Compiler {
    // Add a new instruction.
    pub fn add_instruction(&mut self, instr: Instruction) {
        self.instructions.push(instr);
    }

    pub fn add_string_instruction<S>(&mut self, string: S)
    where
        S: Into<RcString>,
    {
        let index = self.pool.len();
        self.add_instruction(Instruction::String(index));
        self.pool.push(string.into().into());
    }

    pub fn add_bigint_instruction<B>(&mut self, bigint: B)
    where
        B: Into<RcBigInt>,
    {
        let index = self.pool.len();
        self.add_instruction(Instruction::BigInt(index));
        self.pool.push(bigint.into().into());
    }

    pub fn add_defvar_instruction<N, V>(&mut self, name: N, value: V)
    where
        N: Into<RcString>,
        V: Into<Value>,
    {
        let index = self.pool.len();
        self.add_instruction(Instruction::DefVar(index, index + 1));
        self.pool.push(name.into().into());
        self.pool.push(value.into().into());
    }
}

pub(crate) trait CodeGen {
    fn compile(&self, compiler: &mut Compiler, context: &mut Context);
}

impl CodeGen for Node {
    fn compile(&self, compiler: &mut Compiler, context: &mut Context) {
        let _timer = BoaProfiler::global().start_event(&format!("Node ({})", &self), "codeGen");
        match *self {
            Node::Const(Const::Undefined) => compiler.add_instruction(Instruction::Undefined),
            Node::Const(Const::Null) => compiler.add_instruction(Instruction::Null),
            Node::Const(Const::Bool(true)) => compiler.add_instruction(Instruction::True),
            Node::Const(Const::Bool(false)) => compiler.add_instruction(Instruction::False),
            Node::Const(Const::Num(num)) => compiler.add_instruction(Instruction::Rational(num)),
            Node::Const(Const::Int(num)) => match num {
                0 => compiler.add_instruction(Instruction::Zero),
                1 => compiler.add_instruction(Instruction::One),
                _ => compiler.add_instruction(Instruction::Int32(num)),
            },
            Node::Const(Const::String(ref string)) => {
                compiler.add_string_instruction(string.clone())
            }
            Node::Const(Const::BigInt(ref bigint)) => {
                compiler.add_bigint_instruction(bigint.clone())
            }
            Node::BinOp(ref op) => op.compile(compiler, context),
            Node::UnaryOp(ref op) => op.compile(compiler, context),
            Node::LetDeclList(ref list) => {
                for let_decl in list.as_ref() {
                    let value = match let_decl.init() {
                        Some(v) => v.run(context).unwrap_or_else(|_| Value::undefined()),
                        None => Value::undefined(),
                    };

                    compiler.add_defvar_instruction(let_decl.name(), value);
                }
            }
            _ => unimplemented!(),
        }
    }
}
