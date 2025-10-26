use std::io::Write;

pub struct ArgonfileTrailerWriter;

impl ArgonfileTrailerWriter {
    pub fn write<W: Write>(w: &mut W) {
        todo!()
        // 1. Write summary block pointer
        // 2. Write stats block pointer
        // 3. Write compression type info
    }
}
