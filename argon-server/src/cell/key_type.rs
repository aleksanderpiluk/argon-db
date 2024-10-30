#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum KeyType {
    Minimum = 0,
    Put = 4,
    Delete = 8,
    DeleteColumn = 16,
    DeleteFamily = 32,
    Maximum = 255,
}

impl TryFrom<u8> for KeyType {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(KeyType::Minimum),
            4 => Ok(KeyType::Put),
            8 => Ok(KeyType::Delete),
            16 => Ok(KeyType::DeleteColumn),
            32 => Ok(KeyType::DeleteFamily),
            255 => Ok(KeyType::Maximum),
            _ => Err("Invalid value trying to convert u8 to CellType enum."),
        }
    }
}
