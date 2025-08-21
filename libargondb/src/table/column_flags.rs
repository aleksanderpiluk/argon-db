struct ColumnFlags(u16);

impl ColumnFlags {
    const DELETED: u16 = 0x01;
    const IS_NULL: u16 = 0x02;
}
