use log::*;

// Public interfaces.
pub enum LoggerModuleFilterKey {
    Module(&'static str, LevelFilter),
    Default(LevelFilter),
}

pub fn init(
    module_max_levels: impl Into<Vec<LoggerModuleFilterKey>>,
) -> Result<(), SetLoggerError> {
    let module_max_levels = module_max_levels.into();

    set_max_level(most_verbose_level(&module_max_levels));
    set_boxed_logger(Box::new(AppLogger { module_max_levels }))
}

// Internals.
#[inline]
fn most_verbose_level(module_max_levels: &[LoggerModuleFilterKey]) -> LevelFilter {
    let mut most_verbose_level = LevelFilter::Off;
    for level in module_max_levels {
        match level {
            LoggerModuleFilterKey::Module(_, level) | LoggerModuleFilterKey::Default(level) => {
                if *level > most_verbose_level {
                    most_verbose_level = *level;
                }
            }
        }
    }

    most_verbose_level
}

#[inline]
fn level_to_severity_rfc5424(level: Level) -> usize {
    match level {
        Level::Trace => 7,
        Level::Debug => 7,
        Level::Info => 6,
        Level::Warn => 4,
        Level::Error => 3,
    }
}

struct AppLogger {
    module_max_levels: Vec<LoggerModuleFilterKey>,
}
impl Log for AppLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        let mut default_level: Option<LevelFilter> = None;
        for level in &self.module_max_levels {
            match level {
                LoggerModuleFilterKey::Module(name, level) => {
                    if metadata.target().starts_with(name) {
                        return metadata.level().to_level_filter() <= *level;
                    }
                }
                LoggerModuleFilterKey::Default(level) => {
                    if default_level.is_none() {
                        default_level = Some(*level);
                    }
                }
            }
        }

        // Test with default level
        if let Some(default_level) = default_level {
            metadata.level().to_level_filter() <= default_level
        } else {
            false
        }
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!(
                "<{}>{}: {}",
                level_to_severity_rfc5424(record.level()),
                record.target(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

// Tests

#[cfg(test)]
mod tests {
    use log::{Level, LevelFilter, Log, Metadata};

    use crate::{AppLogger, LoggerModuleFilterKey};

    fn metadata<'a>(target: &'a str, level: Level) -> Metadata {
        Metadata::builder().target(target).level(level).build()
    }

    #[test]
    fn most_verbose_level() {
        assert_eq!(
            crate::most_verbose_level(&[
                LoggerModuleFilterKey::Module("test1", LevelFilter::Info),
                LoggerModuleFilterKey::Default(LevelFilter::Error),
            ]),
            LevelFilter::Info
        );
    }

    #[test]
    fn no_filters() {
        let logger = AppLogger {
            module_max_levels: vec![],
        };
        assert_eq!(logger.enabled(&metadata("test1", Level::Error)), false);
    }

    #[test]
    fn no_default_filters() {
        let logger = AppLogger {
            module_max_levels: vec![LoggerModuleFilterKey::Module("test1", LevelFilter::Info)],
        };
        assert_eq!(logger.enabled(&metadata("test1", Level::Info)), true);
        assert_eq!(logger.enabled(&metadata("test1", Level::Trace)), false);

        assert_eq!(logger.enabled(&metadata("test2", Level::Info)), false);
        assert_eq!(logger.enabled(&metadata("test2", Level::Trace)), false);
    }

    #[test]
    fn with_filters() {
        let logger = AppLogger {
            module_max_levels: vec![
                LoggerModuleFilterKey::Module("test1", LevelFilter::Info),
                LoggerModuleFilterKey::Default(LevelFilter::Warn),
            ],
        };
        assert_eq!(logger.enabled(&metadata("test1", Level::Warn)), true);
        assert_eq!(logger.enabled(&metadata("test1", Level::Info)), true);
        assert_eq!(logger.enabled(&metadata("test1", Level::Trace)), false);

        assert_eq!(logger.enabled(&metadata("test2", Level::Warn)), true);
        assert_eq!(logger.enabled(&metadata("test2", Level::Info)), false);
        assert_eq!(logger.enabled(&metadata("test2", Level::Trace)), false);
    }

    #[test]
    fn filter_first_find() {
        let logger = AppLogger {
            module_max_levels: vec![
                LoggerModuleFilterKey::Module("test1", LevelFilter::Error),
                LoggerModuleFilterKey::Module("test1", LevelFilter::Trace),
                LoggerModuleFilterKey::Default(LevelFilter::Error),
                LoggerModuleFilterKey::Default(LevelFilter::Trace),
            ],
        };
        assert_eq!(logger.enabled(&metadata("test1", Level::Error)), true);
        assert_eq!(logger.enabled(&metadata("test1", Level::Trace)), false);

        assert_eq!(logger.enabled(&metadata("test2", Level::Error)), true);
        assert_eq!(logger.enabled(&metadata("test2", Level::Trace)), false);
    }

    #[test]
    fn target_child_module() {
        let logger = AppLogger {
            module_max_levels: vec![LoggerModuleFilterKey::Module("test1", LevelFilter::Error)],
        };
        assert_eq!(
            logger.enabled(&metadata("test1::child", Level::Error)),
            true
        );
    }
}
