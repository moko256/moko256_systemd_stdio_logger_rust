## moko256_systemd_stdio_logger
Simple stdio logger for systemd service.

The systemd will automatically detect log level from stdio. See: http://0pointer.de/blog/projects/journal-submit.html


### Example
application: INFO, other libraries: WARN

#### Code
```toml
[dependencies]
log = { version = "0.4", features = ["max_level_info"]}
moko256_systemd_stdio_logger = { git = "https://github.com/moko256/moko256_systemd_stdio_logger_rust.git", tag = "v1.0.0" }
```

```rust
use moko256_systemd_stdio_logger as logger;

fn main() {
    logger::init([
        logger::LoggerModuleFilterKey::Module(module_path!(), log::LevelFilter::Info),
        logger::LoggerModuleFilterKey::Default(log::LevelFilter::Warn),
    ])
    .unwrap();

    log::info!("Hello, World!");
}
```

#### Output
```
<6>example2: Hello, World!
```

### License
SPDX-License-Identifier: MIT
