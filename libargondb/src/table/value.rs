struct Value();

impl Value {
    fn size(&self) -> ValueSize {
        todo!()
    }

    fn data(&self) -> &[u8] {
        todo!()
    }
}

type ValueSize = u64;
