use std::sync::OnceLock;

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
    set_logger(APP_LOGGER.get_or_init(|| AppLogger { module_max_levels }))
}

// Internals.

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

    most_verbose_level.clone()
}

static APP_LOGGER: OnceLock<AppLogger> = OnceLock::new();
struct AppLogger {
    module_max_levels: Vec<LoggerModuleFilterKey>,
}
impl AppLogger {
    fn level_to_severity_rfc5424(level: Level) -> usize {
        match level {
            Level::Trace => 7,
            Level::Debug => 7,
            Level::Info => 6,
            Level::Warn => 4,
            Level::Error => 3,
        }
    }
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
                    if default_level == None {
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
                AppLogger::level_to_severity_rfc5424(record.level()),
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
        assert_eq!(
            logger.enabled(
                &Metadata::builder()
                    .target("test1")
                    .level(Level::Error)
                    .build()
            ),
            false
        );
    }

    #[test]
    fn no_default_filters() {
        let logger = AppLogger {
            module_max_levels: vec![LoggerModuleFilterKey::Module("test1", LevelFilter::Info)],
        };
        assert_eq!(
            logger.enabled(
                &Metadata::builder()
                    .target("test1")
                    .level(log::Level::Info)
                    .build()
            ),
            true
        );
        assert_eq!(
            logger.enabled(
                &Metadata::builder()
                    .target("test1")
                    .level(log::Level::Trace)
                    .build()
            ),
            false
        );

        assert_eq!(
            logger.enabled(
                &Metadata::builder()
                    .target("test2")
                    .level(log::Level::Info)
                    .build()
            ),
            false
        );
        assert_eq!(
            logger.enabled(
                &Metadata::builder()
                    .target("test2")
                    .level(log::Level::Trace)
                    .build()
            ),
            false
        );
    }

    #[test]
    fn with_filters() {
        let logger = AppLogger {
            module_max_levels: vec![
                LoggerModuleFilterKey::Module("test1", LevelFilter::Info),
                LoggerModuleFilterKey::Default(LevelFilter::Warn),
            ],
        };
        assert_eq!(
            logger.enabled(
                &Metadata::builder()
                    .target("test1")
                    .level(log::Level::Warn)
                    .build()
            ),
            true
        );
        assert_eq!(
            logger.enabled(
                &Metadata::builder()
                    .target("test1")
                    .level(log::Level::Info)
                    .build()
            ),
            true
        );
        assert_eq!(
            logger.enabled(
                &Metadata::builder()
                    .target("test1")
                    .level(log::Level::Trace)
                    .build()
            ),
            false
        );

        assert_eq!(
            logger.enabled(
                &Metadata::builder()
                    .target("test2")
                    .level(log::Level::Warn)
                    .build()
            ),
            true
        );
        assert_eq!(
            logger.enabled(
                &Metadata::builder()
                    .target("test2")
                    .level(log::Level::Info)
                    .build()
            ),
            false
        );
        assert_eq!(
            logger.enabled(
                &Metadata::builder()
                    .target("test2")
                    .level(log::Level::Trace)
                    .build()
            ),
            false
        );
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
        assert_eq!(
            logger.enabled(
                &Metadata::builder()
                    .target("test1")
                    .level(log::Level::Error)
                    .build()
            ),
            true
        );
        assert_eq!(
            logger.enabled(
                &Metadata::builder()
                    .target("test1")
                    .level(log::Level::Trace)
                    .build()
            ),
            false
        );

        assert_eq!(
            logger.enabled(
                &Metadata::builder()
                    .target("test2")
                    .level(log::Level::Error)
                    .build()
            ),
            true
        );
        assert_eq!(
            logger.enabled(
                &Metadata::builder()
                    .target("test2")
                    .level(log::Level::Trace)
                    .build()
            ),
            false
        );
    }

    #[test]
    fn target_child_module() {
        let logger = AppLogger {
            module_max_levels: vec![LoggerModuleFilterKey::Module("test1", LevelFilter::Error)],
        };
        assert_eq!(
            logger.enabled(
                &Metadata::builder()
                    .target("test1::child::module")
                    .level(log::Level::Error)
                    .build()
            ),
            true
        );
    }
}
