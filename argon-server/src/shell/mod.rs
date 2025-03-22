use std::{
    io::{self, Read, Write},
    process,
};

use crate::{
    operations::{ListKeyspacesData, Operation},
    Database,
};

pub struct Shell {}

impl Shell {
    pub const PS1: &str = "argondb> ";

    pub fn read_line() -> Result<String, io::Error> {
        print!("{}", Shell::PS1);
        io::stdout().flush()?;

        let mut input: String = String::new();
        io::stdin().read_line(&mut input)?;

        Ok(input.trim().to_string())
    }

    pub fn execute_aql(input: String) {
        match input.as_str() {
            "show keyspaces;" => {
                let op = Operation::ListKeyspaces(
                    ListKeyspacesData {},
                    Box::new(|r| println!("Result: {:?}", r)),
                );
                let op_id = Database::operations_store().add_op(op);
                Database::channels().operations().push(op_id);
            }
            "exit;" => process::exit(0),
            _ => println!("SYNTAX ERROR"),
        }
    }
}
