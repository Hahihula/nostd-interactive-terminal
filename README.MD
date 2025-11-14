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
//! Basic terminal example for ESP32-C3
//! 
//! This example demonstrates the simplest use of embedded-term
//! with USB Serial JTAG on ESP32-C3.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, signal::Signal};

use esp_hal::{
    Async,
    usb_serial_jtag::UsbSerialJtag,
    interrupt::software::SoftwareInterruptControl,
    timer::timg::TimerGroup,
};
use nostd_interactive_terminal::prelude::*;
use esp_backtrace as _;

// This creates a default app-descriptor required by the esp-idf bootloader.
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(_spawner: Spawner) {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    
    // Split USB Serial JTAG into RX and TX
    let (mut rx, mut tx) = UsbSerialJtag::new(peripherals.USB_DEVICE)
        .into_async()
        .split();
    
    // Create terminal configuration
    let config = TerminalConfig {
        buffer_size: 128,
        prompt: "esp32c3> ",
        echo: true,
        ansi_enabled: true,
    };

    let sw_int = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, sw_int.software_interrupt0);
    
    // Create terminal reader with history
    let history = History::new(nostd_interactive_terminal::HistoryConfig::default());
    let mut reader = nostd_interactive_terminal::terminal::TerminalReader:: <128> ::new(config, Some(history));
    let mut writer = TerminalWriter::new(&mut tx, true);
    
    // Welcome message
    writer.clear_screen().await.unwrap();
    writer.writeln("=== ESP32-C3 Basic Terminal ===\r").await.unwrap();
    writer.writeln("Type 'help' for available commands\r").await.unwrap();
    writer.writeln("\r").await.unwrap();
    
    // Main command loop
    loop {
        match reader.read_line(&mut rx, &mut writer, Option::<&Signal<NoopRawMutex, ()>>::None).await {
            Ok(command) => {
                // Parse command
                let parsed = match CommandParser::parse_simple::<4, 128>(&command) {
                    Ok(p) => p,
                    Err(_) => {
                        writer.write_error("Failed to parse command\r\n").await.unwrap();
                        continue;
                    }
                };
                
                // Handle commands
                match parsed.name() {
                    "help" => {
                        writer.writeln("Available commands:\r").await.unwrap();
                        writer.writeln("  help      - Show this message\r").await.unwrap();
                        writer.writeln("  echo      - Echo back arguments\r").await.unwrap();
                        writer.writeln("  clear     - Clear the screen\r").await.unwrap();
                        writer.writeln("  info      - Show system information\r").await.unwrap();
                    }
                    "echo" => {
                        if let Some(args) = parsed.args_joined(" ") {
                            writer.writeln(&args).await.unwrap();
                            writer.writeln("\r").await.unwrap();
                        } else {
                            writer.write_error("Echo requires arguments\r\n").await.unwrap();
                        }
                    }
                    "clear" => {
                        writer.clear_screen().await.unwrap();
                    }
                    "info" => {
                        writer.writeln("System Information:\r").await.unwrap();
                        writer.writeln("  Device: ESP32-C3\r").await.unwrap();
                        writer.writeln("  Interface: USB Serial JTAG\r").await.unwrap();
                        writer.writeln("  Framework: Embassy (async)\r").await.unwrap();
                    }
                    _ => {
                        {
                            let mut msg: heapless::String<64> = heapless::String::new();
                            msg.push_str("Unknown command: '").unwrap();
                            msg.push_str(parsed.name()).unwrap();
                            msg.push_str("'\r\n").unwrap();
                            writer.write_error(&msg).await.unwrap();
                        }
                        writer.writeln("Type 'help' for available commands\r").await.unwrap();
                    }
                }
            }
            Err(_) => {
                writer.write_error("Error reading line\r\n").await.unwrap();
            }
        }
    }
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

To be tested on:
- ✅ ESP32-S3 (USB Serial JTAG, UART)
- ✅ ESP32 (UART)
- 🔄 Other Embassy-supported platforms (should work, not tested)

## License

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)


## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

Inspired by terminal implementations in embedded Rust projects and designed specifically for Embassy-based async applications.