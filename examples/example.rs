use logger::LoggerModuleFilterKey;
use moko256_systemd_stdio_logger as logger;

fn main() {
    logger::init([
        LoggerModuleFilterKey::Module("example::mod1", log::LevelFilter::Error),
        LoggerModuleFilterKey::Default(log::LevelFilter::Trace),
    ])
    .unwrap();

    log::trace!("Hello, World!");
    mod1::do_something();
}

mod mod1 {
    pub fn do_something() {
        log::trace!("Very very very verbose log. (will not shown)");
        log::error!("Very important error. (will shown)");
    }
}
