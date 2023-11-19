use std::{io, mem};

pub const TAPE_CHUNK_SIZE: usize = 8192;

pub struct Interface {
    input: Box<dyn io::Read + Send + Sync + 'static>,
    output: Box<dyn io::Write + Send + Sync + 'static>,
}

impl Interface {
    pub fn new<R, W>(input: R, output: W) -> Self
    where
        R: io::Read + Send + Sync + 'static,
        W: io::Write + Send + Sync + 'static,
    {
        Self { input: Box::new(input), output: Box::new(output) }
    }
}

pub unsafe extern "sysv64" fn put(interface: *mut Interface, ch: u8) -> i8 {
    if (*interface).output.write_all(&[ch]).is_ok() {
        0
    } else {
        -1
    }
}

pub unsafe extern "sysv64" fn get(interface: *mut Interface) -> i16 {
    let mut buf = [0];
    match (*interface).input.read_exact(&mut buf) {
        Ok(_) => (1 << 8) | (buf[0] as i16),
        Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => 0,
        Err(_) => -1,
    }
}

pub unsafe extern "sysv64" fn create_tape() -> *mut u8 {
    libc::calloc(TAPE_CHUNK_SIZE, mem::size_of::<u8>()) as *mut u8
}

pub unsafe extern "sysv64" fn destroy_tape(tape: *mut u8) {
    libc::free(tape as *mut libc::c_void)
}

pub unsafe extern "sysv64" fn grow_next(
    tape_start: *mut u8,
    tape_len: usize,
) -> *mut u8 {
    let new_len = tape_len + TAPE_CHUNK_SIZE;
    let new_start =
        libc::realloc(tape_start as *mut libc::c_void, new_len) as *mut u8;
    if new_start.is_null() {
        panic!("could not reallocate tape");
    }
    libc::memset(
        new_start.add(tape_len) as *mut libc::c_void,
        0,
        TAPE_CHUNK_SIZE,
    );
    new_start
}

pub unsafe extern "sysv64" fn grow_prev(
    tape_start: *mut u8,
    tape_len: usize,
) -> *mut u8 {
    let new_len = tape_len + TAPE_CHUNK_SIZE;
    let new_start =
        libc::realloc(tape_start as *mut libc::c_void, new_len) as *mut u8;
    if new_start.is_null() {
        panic!("could not reallocate tape");
    }
    libc::memmove(
        new_start.add(TAPE_CHUNK_SIZE) as *mut libc::c_void,
        new_start as *mut libc::c_void,
        tape_len,
    );
    libc::memset(new_start as *mut libc::c_void, 0, TAPE_CHUNK_SIZE);
    new_start
}
