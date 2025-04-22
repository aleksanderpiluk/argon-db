pub struct MutationExecutionThread {}

impl MutationExecutionThread {
    pub fn init() -> impl Fn() {
        return || println!("HELLO WORLD from thread!!!");
    }
}
