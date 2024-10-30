use log4rs::{
    append::console::ConsoleAppender,
    config::{Appender, Root},
    Config,
};

pub struct LogCtl {
    handle: log4rs::Handle,
}

impl LogCtl {
    pub fn bootstrap() -> Self {
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
