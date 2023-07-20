use crate::bytecode::{Bytecode, Opcode};

use std::collections::HashMap;

pub const STYLE_RED_BOLD: &str = "\x1b[31;1m";
pub const STYLE_YELLOW: &str = "\x1b[33m";
pub const STYLE_CYAN_BOLD: &str = "\x1b[36;1m";
// pub const STYLE_MAGENTA_BRIGHT: &str = "\x1b[95m";
pub const STYLE_DIM: &str = "\x1b[2m";
pub const STYLE_RESET: &str = "\x1b[0m";

fn const_instruction(
    instruction: &str,
    is_short: bool,
    bc: &Bytecode,
    offset: usize,
    strings: &HashMap<u64, String>,
) -> usize {
    let const_offset = if is_short {
        ((bc.byte_at(offset + 1) as u16) << 8) | bc.byte_at(offset + 2) as u16
    } else {
        bc.byte_at(offset + 1) as u16
    };
    let constant = bc.const_at(const_offset as usize);

    print!("{STYLE_CYAN_BOLD}{instruction:<24}{STYLE_RESET}");
    print!("{STYLE_DIM}{const_offset:>4}{STYLE_RESET} ");

    print!("{STYLE_DIM}`{STYLE_RESET}");
    constant.print(strings);
    println!("{STYLE_DIM}`{STYLE_RESET}");

    if is_short {
        offset + 3
    } else {
        offset + 2
    }
}

fn byte_instruction(instruction: &str, bc: &Bytecode, offset: usize) -> usize {
    let local_offset = bc.byte_at(offset + 1);

    print!("{STYLE_CYAN_BOLD}{instruction:<24}{STYLE_RESET}");
    println!("{STYLE_DIM}{local_offset:>4}{STYLE_RESET}");

    offset + 2
}

fn jump_instruction(instruction: &str, bc: &Bytecode, offset: usize, sign: i32) -> usize {
    let jump = ((bc.byte_at(offset + 1) as u16) << 8) | bc.byte_at(offset + 2) as u16;

    print!("{STYLE_CYAN_BOLD}{instruction:<27}{STYLE_RESET}");
    println!(
        "{STYLE_DIM}──>{STYLE_RESET}{:0>4}",
        jump as i32 * sign + offset as i32 + 3
    );

    offset + 3
}

pub fn disassemble_instruction(
    bc: &Bytecode,
    offset: usize,
    strings: &HashMap<u64, String>,
) -> usize {
    print!("{STYLE_DIM}{offset:0>4}{STYLE_RESET} ");
    if offset > 0 && bc.line_num_at(offset) == bc.line_num_at(offset - 1) {
        print!("   {STYLE_DIM}⋮{STYLE_RESET} ");
    } else {
        print!("{STYLE_DIM}{: >4}{STYLE_RESET} ", bc.line_num_at(offset),);
    }

    let simple_instruction = |instruction, offset| {
        println!("{STYLE_CYAN_BOLD}{instruction}{STYLE_RESET}");
        offset + 1
    };
    let opcode = bc.byte_at(offset);

    match opcode {
        b if b == Opcode::Constant as u8 || b == Opcode::ConstantShort as u8 => {
            let is_short = b == Opcode::ConstantShort as u8;
            let instruction = if is_short {
                "ConstantShort"
            } else {
                "Constant"
            };

            const_instruction(instruction, is_short, bc, offset, strings)
        }

        b if b == Opcode::DefineGlobal as u8 || b == Opcode::DefineGlobalShort as u8 => {
            let is_short = b == Opcode::DefineGlobalShort as u8;
            let instruction = if is_short {
                "DefineGlobalShort"
            } else {
                "DefineGlobal"
            };

            const_instruction(instruction, is_short, bc, offset, strings)
        }

        b if b == Opcode::GetGlobal as u8 || b == Opcode::GetGlobalShort as u8 => {
            let is_short = b == Opcode::GetGlobalShort as u8;
            let instruction = if is_short {
                "GetGlobalShort"
            } else {
                "GetGlobal"
            };

            const_instruction(instruction, is_short, bc, offset, strings)
        }

        b if b == Opcode::SetGlobal as u8 || b == Opcode::SetGlobalShort as u8 => {
            let is_short = b == Opcode::SetGlobalShort as u8;
            let instruction = if is_short {
                "SetGlobalShort"
            } else {
                "SetGlobal"
            };

            const_instruction(instruction, is_short, bc, offset, strings)
        }

        b if b == Opcode::NativeFn as u8 || b == Opcode::NativeFnShort as u8 => {
            let is_short = b == Opcode::NativeFnShort as u8;
            let instruction = if is_short {
                "NativeFnShort"
            } else {
                "NativeFn"
            };

            const_instruction(instruction, is_short, bc, offset, strings)
        }

        b if b == Opcode::Call as u8 => byte_instruction("Call", bc, offset),

        b if b == Opcode::GetLocal as u8 => byte_instruction("GetLocal", bc, offset),
        b if b == Opcode::SetLocal as u8 => byte_instruction("SetLocal", bc, offset),

        b if b == Opcode::Jump as u8 => jump_instruction("Jump", bc, offset, 1),
        b if b == Opcode::JumpIfFalse as u8 => jump_instruction("JumpIfFalse", bc, offset, 1),
        b if b == Opcode::Loop as u8 => jump_instruction("Loop", bc, offset, -1),
        b if b == Opcode::Pass as u8 => simple_instruction("Pass", offset),

        b if b == Opcode::Add as u8 => simple_instruction("Add", offset),
        b if b == Opcode::Negate as u8 => simple_instruction("Negate", offset),
        b if b == Opcode::Null as u8 => simple_instruction("Null", offset),
        b if b == Opcode::LineFeed as u8 => simple_instruction("LineFeed", offset),

        b if b == Opcode::True as u8 => simple_instruction("True", offset),
        b if b == Opcode::False as u8 => simple_instruction("False", offset),
        b if b == Opcode::Not as u8 => simple_instruction("Not", offset),

        b if b == Opcode::Equal as u8 => simple_instruction("Equal", offset),
        b if b == Opcode::NotEqual as u8 => simple_instruction("NotEqual", offset),
        b if b == Opcode::Less as u8 => simple_instruction("Less", offset),
        b if b == Opcode::LessEqual as u8 => simple_instruction("LessEqual", offset),
        b if b == Opcode::Greater as u8 => simple_instruction("Greater", offset),
        b if b == Opcode::GreaterEqual as u8 => simple_instruction("GreaterEqual", offset),

        b if b == Opcode::Pop as u8 => simple_instruction("Pop", offset),
        b if b == Opcode::Write as u8 => simple_instruction("Write", offset),
        b if b == Opcode::Return as u8 => simple_instruction("Return", offset),

        _ => {
            println!("{STYLE_RED_BOLD}Undefined opcode: {opcode}{STYLE_RESET}");

            offset + 1
        }
    }
}

pub fn disassemble_bytecode(bc: &Bytecode, heading: &str, strings: &HashMap<u64, String>) {
    print!("{:─<9} ", "");
    print!("{heading}");
    println!(" {:─<18}", "");

    let mut offset = 0;
    while offset < bc.byte_count() {
        offset = disassemble_instruction(bc, offset, strings);
    }

    println!("{:─<44}", "");
    println!();
}
