use log4rs::{
    Config,
    append::console::ConsoleAppender,
    config::{Appender, Root},
};

pub struct LogSystem {
    handle: log4rs::Handle,
}

impl LogSystem {
    pub fn init() -> Self {
        let stdout = ConsoleAppender::builder().build();

        let config = Config::builder()
            .appender(Appender::builder().build("stdout", Box::new(stdout)))
            .build(
                Root::builder()
                    .appender("stdout")
                    .build(log::LevelFilter::max()),
            )
            .unwrap();

        let handle = log4rs::init_config(config).unwrap();

        Self { handle }
    }
}

// log_file: String::from("/var/log/argon-db.log"),
