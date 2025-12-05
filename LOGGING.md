# Logging System

## Overview

The application now uses `log` + `env_logger` for structured logging instead of `eprintln!`/`println!`.

## Log Levels

- **ERROR** - Critical failures (shader validation errors, audio errors)
- **WARN** - Warnings (audio initialization failures)
- **INFO** - Important events (app start, shader compilation success, audio loaded)
- **DEBUG** - Detailed flow (shader parsing, audio threads, first render)
- **TRACE** - Very detailed (uniform buffer creation, audio loops)

## Running with Logs

```bash
# Info level (default)
RUST_LOG=info cargo run

# Debug level (more detail)
RUST_LOG=debug cargo run

# Trace level (everything)
RUST_LOG=trace cargo run

# Specific modules
RUST_LOG=egui_two_windows::utils::pipeline=debug cargo run
```

## Key Log Points

### Application Lifecycle
- `INFO` - Application starting
- `INFO` - TopApp initialization
- `INFO` - Application terminated normally

### Shader Pipeline
- `DEBUG` - Creating shader pipeline
- `DEBUG` - Naga validation
- `INFO` - Pipeline created successfully
- `ERROR` - Parse/validation failures

### Audio System
- `INFO` - Loading audio file
- `INFO` - Audio format details
- `DEBUG` - FFT analyzer thread lifecycle
- `DEBUG` - Playback thread lifecycle
- `WARN` - Audio initialization failures

### User Actions
- `INFO` - Apply shader requested
- `INFO` - Resetting shader to defaults

## Benefits

✅ No more cluttered console output  
✅ Configurable verbosity  
✅ Production-ready logging  
✅ Easy debugging with RUST_LOG  
✅ Clean separation of concerns
