# nostd-interactive-terminal

[![Crates.io](https://img.shields.io/crates/v/nostd-interactive-terminal.svg)](https://crates.io/crates/nostd-interactive-terminal)
[![Documentation](https://docs.rs/nostd-interactive-terminal/badge.svg)](https://docs.rs/nostd-interactive-terminal)

A `no_std` interactive terminal library for embedded systems with line editing, command history, and command parsing capabilities.

## Features

- 📝 **Line Editing**: Full line editing with cursor movement, backspace, delete
- 📚 **Command History**: Navigate through previously entered commands
- 🎨 **ANSI Support**: Optional ANSI escape codes for colored output and better terminal control
- 🔧 **Command Parsing**: Built-in parser for splitting commands and arguments
- ⚡ **Async/Await**: Built on Embassy's async runtime for efficient multitasking
- 🎯 **Flexible**: Works with any `embedded-io-async` compatible UART
- 🔒 **Type-Safe**: Compile-time buffer sizes with `heapless`
- 🚫 **No Allocations**: Pure `no_std` with no heap allocations required

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
nostd-interactive-terminal = "0.1"
heapless = "0.8"
```

## Basic Usage

```rust
use nostd_interactive_terminal::prelude::*;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::signal::Signal;

// Create terminal configuration
let config = TerminalConfig {
    buffer_size: 128,
    prompt: "> ",
    echo: true,
    ansi_enabled: true,
};

// Create terminal reader with history
let history = History::new(HistoryConfig::default());
let mut reader = TerminalReader::<128>::new(config, Some(history));

// Create writer for output
let mut writer = TerminalWriter::new(&mut uart_tx, true);

// Read commands in a loop
loop {
    match reader.read_line(&mut uart_rx, &mut writer, None).await {
        Ok(command) => {
            // Parse the command
            let parsed = CommandParser::parse_simple::<8, 128>(&command).unwrap();
            
            match parsed.name() {
                "help" => {
                    writer.writeln("Available commands:").await.unwrap();
                    writer.writeln("  help - Show this message").await.unwrap();
                }
                "hello" => {
                    writer.write_success("Hello, World!\r\n").await.unwrap();
                }
                _ => {
                    writer.write_error("Unknown command\r\n").await.unwrap();
                }
            }
        }
        Err(_) => break,
    }
}
```

## ESP32 Example

Complete example for ESP32-C3 with USB Serial JTAG:

```rust
#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex, signal::Signal};
use esp_hal::{
    Async, clock::CpuClock,
    usb_serial_jtag::{UsbSerialJtag, UsbSerialJtagRx, UsbSerialJtagTx},
};
use nostd_interactive_terminal::prelude::*;
use static_cell::StaticCell;

#[embassy_executor::task]
async fn terminal_task(
    mut rx: UsbSerialJtagRx<'static, Async>,
    tx_mutex: &'static Mutex<NoopRawMutex, UsbSerialJtagTx<'static, Async>>,
    redraw_signal: &'static Signal<NoopRawMutex, ()>,
) {
    let config = TerminalConfig::default();
    let history = History::new(HistoryConfig::default());
    let mut reader = TerminalReader::<128>::new(config, Some(history));

    loop {
        let mut tx = tx_mutex.lock().await;
        let mut writer = TerminalWriter::new(&mut *tx, true);
        
        match reader.read_line(&mut rx, &mut writer, Some(redraw_signal)).await {
            Ok(command) => {
                // Handle command
                drop(tx); // Release lock before processing
                // ... process command ...
            }
            Err(_) => break,
        }
    }
}

#[esp_hal::main]
async fn main(spawner: Spawner) {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    
    let (rx, tx) = UsbSerialJtag::new(peripherals.USB_DEVICE)
        .into_async()
        .split();
    
    static TX_MUTEX: StaticCell<Mutex<NoopRawMutex, UsbSerialJtagTx<'static, Async>>> = 
        StaticCell::new();
    let tx_mutex = &*TX_MUTEX.init(Mutex::new(tx));
    
    static REDRAW_SIGNAL: StaticCell<Signal<NoopRawMutex, ()>> = StaticCell::new();
    let redraw_signal = &*REDRAW_SIGNAL.init(Signal::new());
    
    spawner.spawn(terminal_task(rx, tx_mutex, redraw_signal)).unwrap();
}
```

## Features

### Line Editing

The terminal supports standard line editing features:
- **Backspace/Delete**: Remove characters
- **Arrow Keys**: Move cursor (when ANSI enabled)
- **Ctrl+C**: Interrupt current line
- **Ctrl+D**: End of file signal

### Command History

Navigate through previous commands:
- **Up Arrow**: Previous command
- **Down Arrow**: Next command
- Configurable history size
- Optional deduplication of consecutive identical commands

### Command Parsing

Multiple parsing strategies:

```rust
// Simple whitespace split
let cmd = CommandParser::parse_simple::<8, 128>("send 192.168.1.1 hello");

// Quote-aware parsing
let cmd = CommandParser::parse::<8, 128>(r#"send peer "hello world""#);

// Limited splits (remaining text in last arg)
let cmd = CommandParser::parse_max_split::<8, 128>("broadcast this is a message", 1);
```

### ANSI Support

When enabled, provides:
- Colored output (error, success, warning, info)
- Screen clearing
- Cursor movement
- Text formatting (bold, colors)

```rust
writer.write_error("Error: Invalid command\r\n").await?;
writer.write_success("Command executed successfully\r\n").await?;
writer.write_colored("Custom color text", colors::CYAN).await?;
```

### Redraw Signal

Support for async redrawing when other tasks print output:

```rust
// Task that prints messages
#[embassy_executor::task]
async fn message_printer(
    tx_mutex: &'static Mutex<NoopRawMutex, UartTx>,
    redraw_signal: &'static Signal<NoopRawMutex, ()>,
) {
    loop {
        {
            let mut tx = tx_mutex.lock().await;
            let mut writer = TerminalWriter::new(&mut *tx, true);
            writer.clear_line().await.unwrap();
            writer.writeln("Incoming message!").await.unwrap();
        }
        redraw_signal.signal(()); // Trigger prompt redraw
        Timer::after_secs(5).await;
    }
}
```

## Configuration

### Terminal Config

```rust
let config = TerminalConfig {
    buffer_size: 128,        // Max command length
    prompt: "$ ",            // Prompt string
    echo: true,              // Echo typed characters
    ansi_enabled: true,      // Use ANSI escape codes
};
```

### History Config

```rust
let history_config = HistoryConfig {
    max_entries: 20,         // Max history entries
    deduplicate: true,       // Skip duplicate consecutive commands
};
```

## Platform Support

This crate is designed to work with any embedded platform that supports:
- `no_std` environment
- `embedded-io-async` traits
- Embassy async runtime

Tested on:
- ✅ ESP32-C3 (USB Serial JTAG)
- ✅ ESP32-S3 (USB Serial JTAG, UART)
- ✅ ESP32 (UART)
- 🔄 Other Embassy-supported platforms (should work, not tested)

## Examples

See the `examples/` directory for complete examples:
- `esp32c3_basic.rs` - Basic terminal for ESP32-C3
- `esp32c3_full.rs` - Full-featured terminal with ESP-NOW integration
- `command_handlers.rs` - Example command handler implementation

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

Inspired by terminal implementations in embedded Rust projects and designed specifically for Embassy-based async applications.