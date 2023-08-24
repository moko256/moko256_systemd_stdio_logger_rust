use moko256_systemd_stdio_logger as logger;

fn main() {
    logger::init([
        logger::LoggerModuleFilterKey::Module(module_path!(), log::LevelFilter::Info),
        logger::LoggerModuleFilterKey::Default(log::LevelFilter::Warn),
    ])
    .unwrap();

    log::info!("Hello, World!");
}
