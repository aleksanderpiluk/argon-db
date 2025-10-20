#[derive(Debug, Clone, Copy)]
pub enum KVConstructorError {
    InvalidData,
}

#[derive(Debug, Clone, Copy)]
pub enum KVRuntimeError {
    IndexOutOfBounds,
    DataMalformed,
}
