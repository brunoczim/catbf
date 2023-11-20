use self::runtime::Interface;
use crate::ir::{Instruction, Program};
use std::{
    collections::{BTreeMap, HashMap},
    io,
    mem::transmute,
    ptr,
};
use thiserror::Error;

mod runtime;

pub const TARGET_SUPPORTED: bool =
    cfg!(all(target_os = "linux", target_arch = "x86_64"));

const PUSH_RBX: [u8; 1] = [0x53];
const PUSH_R12: [u8; 2] = [0x41, 0x54];
const PUSH_R13: [u8; 2] = [0x41, 0x55];
const PUSH_R14: [u8; 2] = [0x41, 0x56];

const POP_R14: [u8; 2] = [0x41, 0x5e];
const POP_R13: [u8; 2] = [0x41, 0x5d];
const POP_R12: [u8; 2] = [0x41, 0x5c];
const POP_RBX: [u8; 1] = [0x5b];

const MOV_RDI_TO_RBX: [u8; 3] = [0x48, 0x89, 0xfb];
const MOV_R12_TO_RDI: [u8; 3] = [0x4c, 0x89, 0xe7];
const MOV_R13_TO_RSI: [u8; 3] = [0x4c, 0x89, 0xee];
const MOV_RAX_TO_R12: [u8; 3] = [0x49, 0x89, 0xc4];
const MOV_RBX_TO_RDI: [u8; 3] = [0x48, 0x89, 0xdf];
const MOV_AX_TO_SI: [u8; 3] = [0x66, 0x89, 0xc6];
const MOV_AX_TO_MEM_R12_R14: [u8; 5] = [0x66, 0x43, 0x89, 0x04, 0x34];
const MOV_R14B_TO_AL: [u8; 3] = [0x44, 0x88, 0xf0];
const MOV_MEM_R12_R14_TO_AL: [u8; 4] = [0x43, 0x8a, 0x04, 0x34];
const MOVABS_TO_RAX: [u8; 2] = [0x48, 0xb8];

const CMP_R14_WITH_R13: [u8; 3] = [0x4d, 0x39, 0xee];
const TEST_R14_WITH_R14: [u8; 3] = [0x4d, 0x85, 0xf6];
const TEST_RAX_WITH_RAX: [u8; 3] = [0x48, 0x85, 0xc0];
const TEST_AX_WITH_AX: [u8; 3] = [0x66, 0x85, 0xc0];
const TEST_AL_WITH_AL: [u8; 2] = [0x84, 0xc0];

const JMP_REL32: [u8; 1] = [0xe9];
const JE_JZ_REL32: [u8; 2] = [0x0f, 0x84];
const JNE_JNZ_REL32: [u8; 2] = [0x0f, 0x85];
const JS_REL32: [u8; 2] = [0x0f, 0x88];
const CALL_ABS_RAX: [u8; 2] = [0xff, 0xd0];

const XOR_R14_TO_R14: [u8; 3] = [0x4d, 0x31, 0xf6];
const XOR_EAX_TO_EAX: [u8; 2] = [0x31, 0xc0];
const XOR_R14B_TO_R14B: [u8; 3] = [0x45, 0x30, 0xf6];

const MOV_IMM32_TO_R13: [u8; 3] = [0x49, 0xc7, 0xc5];
const MOV_IMM8_TO_R14B: [u8; 2] = [0x41, 0xb6];
const ADD_IMM32_TO_R13: [u8; 3] = [0x49, 0x81, 0xc5];
const ADD_IMM32_TO_R14: [u8; 3] = [0x49, 0x81, 0xc6];

const ROR_IMM8_TO_AX: [u8; 3] = [0x66, 0xc1, 0xc8];

const INC_R14: [u8; 3] = [0x49, 0xff, 0xc6];
const DEC_R14: [u8; 3] = [0x49, 0xff, 0xce];

const INCB_MEM_R12_R14: [u8; 4] = [0x43, 0xfe, 0x04, 0x34];
const DECB_MEM_R12_R14: [u8; 4] = [0x43, 0xfe, 0x0c, 0x34];

const RET: [u8; 1] = [0xc3];

pub fn compile(program: &Program) -> Result<Executable, Error> {
    if !TARGET_SUPPORTED {
        Err(Error::UnsupportedTarget)?;
    }

    let mut compiler = Compiler::new();

    compiler.first_pass(program);
    compiler.second_pass()?;

    unsafe { Executable::new(&compiler.buf[..]) }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("target is unsupported for Just-In-Time compilation")]
    UnsupportedTarget,
    #[error("label index {} is out of bounds", .0)]
    BadLabelIndex(usize),
    #[error("could not allocate memory for just in time compilation: {}", .0)]
    AllocError(io::Error),
    #[error("error setting permission for executable memory: {}", .0)]
    Permission(io::Error),
}

#[derive(Debug)]
pub struct Executable {
    buf: *mut libc::c_void,
}

impl Executable {
    unsafe fn new(buf: &[u8]) -> Result<Self, Error> {
        let page_size = libc::sysconf(libc::_SC_PAGESIZE) as libc::size_t;
        let len = buf.len() as libc::size_t;
        let ceiled_len = len + (page_size - len % page_size) % page_size;

        let mut ptr = ptr::null_mut();
        let code = libc::posix_memalign(&mut ptr, page_size, ceiled_len);
        if code != 0 {
            Err(Error::AllocError(io::Error::from_raw_os_error(code)))?;
        }
        libc::memcpy(ptr, buf.as_ptr() as *const libc::c_void, len);

        let protection = libc::PROT_EXEC | libc::PROT_READ;
        if libc::mprotect(ptr, ceiled_len, protection) < 0 {
            Err(Error::Permission(io::Error::last_os_error()))?;
        }

        Ok(Self { buf: ptr })
    }

    pub fn run<R, W>(&self, input: R, output: W) -> io::Result<()>
    where
        R: io::Read + Send + Sync + 'static,
        W: io::Write + Send + Sync + 'static,
    {
        let mut interface = Interface::new(input, output);

        let status = unsafe {
            let main: unsafe fn(*mut Interface) -> i8 = transmute(self.buf);
            main(&mut interface)
        };

        if status < 0 {
            Err(io::Error::last_os_error())?;
        }
        Ok(())
    }
}

impl Drop for Executable {
    fn drop(&mut self) {
        unsafe {
            libc::free(self.buf);
        }
    }
}

#[derive(Debug, Clone)]
struct Compiler {
    buf: Vec<u8>,
    placeholders: BTreeMap<usize, (usize, usize)>,
    labels: HashMap<(usize, usize), usize>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            buf: Vec::new(),
            placeholders: BTreeMap::new(),
            labels: HashMap::new(),
        }
    }

    pub fn first_pass(&mut self, program: &Program) {
        let last_ir_label = program.code.len();
        self.write_enter(last_ir_label);

        for (ir_label, instr) in program.code.iter().enumerate() {
            self.def_main_label(ir_label);
            self.handle_instruction(ir_label, *instr, last_ir_label);
        }

        self.def_main_label(last_ir_label);
        self.write_leave(last_ir_label);
    }

    pub fn second_pass(&mut self) -> Result<(), Error> {
        for (placeholder_label, (ir_label, sub_ir_label)) in &self.placeholders
        {
            let Some(label) = self.labels.get(&(*ir_label, *sub_ir_label))
            else {
                Err(Error::BadLabelIndex(*ir_label))?
            };
            let from = (placeholder_label + 4) as i64;
            let to = *label as i64;
            let distance = to.wrapping_sub(from) as u32;
            let label_buf = distance.to_le_bytes();
            self.buf[*placeholder_label .. *placeholder_label + 4]
                .copy_from_slice(&label_buf[..]);
        }
        Ok(())
    }

    pub fn handle_instruction(
        &mut self,
        ir_label: usize,
        instr: Instruction,
        last_ir_label: usize,
    ) {
        match instr {
            Instruction::Inc => self.write_inc(),
            Instruction::Dec => self.write_dec(),
            Instruction::Next => self.write_next(ir_label, last_ir_label),
            Instruction::Prev => self.write_prev(ir_label, last_ir_label),
            Instruction::Get => self.write_get(ir_label, last_ir_label),
            Instruction::Put => self.write_put(last_ir_label),
            Instruction::Jz(target_ir_label) => self.write_jz(target_ir_label),
            Instruction::Jnz(target_ir_label) => {
                self.write_jnz(target_ir_label)
            },
            Instruction::Halt => self.write_halt(last_ir_label),
        }
    }

    pub fn write<I>(&mut self, bytes: I)
    where
        I: IntoIterator<Item = u8>,
    {
        self.buf.extend(bytes);
    }

    pub fn def_main_label(&mut self, ir_label: usize) {
        self.def_label(ir_label, 0)
    }

    pub fn def_label(&mut self, ir_label: usize, sub_label: usize) {
        self.labels.insert((ir_label, sub_label), self.buf.len());
    }

    pub fn make_placeholder(&mut self, ir_label: usize, sub_label: usize) {
        self.placeholders.insert(self.buf.len(), (ir_label, sub_label));
        self.write(0u32.to_le_bytes());
    }

    pub fn call_absolute(&mut self, func_ptr: *const u8) {
        self.write(MOVABS_TO_RAX);
        self.write((func_ptr as usize as u64).to_le_bytes());
        self.write(CALL_ABS_RAX);
    }

    pub fn write_enter(&mut self, last_ir_label: usize) {
        self.write(PUSH_R14);
        self.write(PUSH_R13);
        self.write(PUSH_R12);
        self.write(PUSH_RBX);
        self.write(MOV_RDI_TO_RBX);
        self.write(XOR_R14_TO_R14);
        self.call_absolute(runtime::create_tape as *const u8);
        self.write(TEST_RAX_WITH_RAX);
        self.write(JE_JZ_REL32);
        self.make_placeholder(last_ir_label, 1);
        self.write(MOV_IMM32_TO_R13);
        self.write((runtime::TAPE_CHUNK_SIZE as u32).to_le_bytes());
        self.write(MOV_RAX_TO_R12);
    }

    pub fn write_leave(&mut self, ir_label: usize) {
        self.write(XOR_R14B_TO_R14B);
        self.write(JMP_REL32);
        self.make_placeholder(ir_label, 2);
        self.def_label(ir_label, 1);
        self.write(MOV_IMM8_TO_R14B);
        self.write((-1i8).to_le_bytes());
        self.def_label(ir_label, 2);
        self.write(MOV_R12_TO_RDI);
        self.call_absolute(runtime::destroy_tape as *const u8);
        self.write(MOV_R14B_TO_AL);
        self.write(POP_RBX);
        self.write(POP_R12);
        self.write(POP_R13);
        self.write(POP_R14);
        self.write(RET);
    }

    pub fn write_inc(&mut self) {
        self.write(INCB_MEM_R12_R14);
    }

    pub fn write_dec(&mut self) {
        self.write(DECB_MEM_R12_R14);
    }

    pub fn write_next(&mut self, ir_label: usize, last_ir_label: usize) {
        self.write(CMP_R14_WITH_R13);
        self.write(JNE_JNZ_REL32);
        self.make_placeholder(ir_label, 1);
        self.write(MOV_R12_TO_RDI);
        self.write(MOV_R13_TO_RSI);
        self.call_absolute(runtime::grow_next as *const u8);
        self.write(TEST_RAX_WITH_RAX);
        self.write(JE_JZ_REL32);
        self.make_placeholder(last_ir_label, 1);
        self.write(MOV_RAX_TO_R12);
        self.write(ADD_IMM32_TO_R13);
        self.write((runtime::TAPE_CHUNK_SIZE as u32).to_le_bytes());
        self.def_label(ir_label, 1);
        self.write(INC_R14);
    }

    pub fn write_prev(&mut self, ir_label: usize, last_ir_label: usize) {
        self.write(TEST_R14_WITH_R14);
        self.write(JNE_JNZ_REL32);
        self.make_placeholder(ir_label, 1);
        self.write(MOV_R12_TO_RDI);
        self.write(MOV_R13_TO_RSI);
        self.call_absolute(runtime::grow_prev as *const u8);
        self.write(TEST_RAX_WITH_RAX);
        self.write(JE_JZ_REL32);
        self.make_placeholder(last_ir_label, 1);
        self.write(ADD_IMM32_TO_R14);
        self.write((runtime::TAPE_CHUNK_SIZE as u32).to_le_bytes());
        self.write(MOV_RAX_TO_R12);
        self.write(ADD_IMM32_TO_R13);
        self.write((runtime::TAPE_CHUNK_SIZE as u32).to_le_bytes());
        self.def_label(ir_label, 1);
        self.write(DEC_R14);
    }

    pub fn write_put(&mut self, last_ir_label: usize) {
        self.write(MOV_RBX_TO_RDI);
        self.write(XOR_EAX_TO_EAX);
        self.write(MOV_MEM_R12_R14_TO_AL);
        self.write(MOV_AX_TO_SI);
        self.call_absolute(runtime::put as *const u8);
        self.write(TEST_AL_WITH_AL);
        self.write(JS_REL32);
        self.make_placeholder(last_ir_label, 1);
    }

    pub fn write_get(&mut self, ir_label: usize, last_ir_label: usize) {
        self.write(CMP_R14_WITH_R13);
        self.write(JNE_JNZ_REL32);
        self.make_placeholder(ir_label, 1);
        self.write(MOV_R12_TO_RDI);
        self.write(MOV_R13_TO_RSI);
        self.call_absolute(runtime::grow_next as *const u8);
        self.write(TEST_RAX_WITH_RAX);
        self.write(JE_JZ_REL32);
        self.make_placeholder(last_ir_label, 1);
        self.write(MOV_RAX_TO_R12);
        self.write(ADD_IMM32_TO_R13);
        self.write((runtime::TAPE_CHUNK_SIZE as u32).to_le_bytes());
        self.def_label(ir_label, 1);
        self.write(MOV_RBX_TO_RDI);
        self.call_absolute(runtime::get as *const u8);
        self.write(TEST_AX_WITH_AX);
        self.write(JS_REL32);
        self.make_placeholder(last_ir_label, 1);
        self.write(ROR_IMM8_TO_AX);
        self.write(8u8.to_le_bytes());
        self.write(MOV_AX_TO_MEM_R12_R14);
    }

    pub fn write_halt(&mut self, last_ir_label: usize) {
        self.write(JMP_REL32);
        self.make_placeholder(last_ir_label, 0);
    }

    pub fn write_jz(&mut self, target_ir_label: usize) {
        self.write(MOV_MEM_R12_R14_TO_AL);
        self.write(TEST_AL_WITH_AL);
        self.write(JE_JZ_REL32);
        self.make_placeholder(target_ir_label, 0);
    }

    pub fn write_jnz(&mut self, target_ir_label: usize) {
        self.write(MOV_MEM_R12_R14_TO_AL);
        self.write(TEST_AL_WITH_AL);
        self.write(JNE_JNZ_REL32);
        self.make_placeholder(target_ir_label, 0);
    }
}
