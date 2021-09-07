# human-panic-logger

Panic messages and logs for humans. Handles panics by calling
[`std::panic::set_hook`](https://doc.rust-lang.org/std/panic/fn.set_hook.html)
to make errors nice for humans.

Compared to the ordinary crate, this one initializes the `log` crate and starts a logfile for you with the supplied file param.
Just use the default log macros `info!()`, and it'll all get put into the file, and this human panic logger will log panics to the log file too.

The logs will log debug level if built in debug mode. If built in release mode, logs will only log info level and above.

## Why?
When you're building a CLI, polish is super important. Even though Rust is
pretty great at safety, it's not unheard of to access the wrong index in a
vector or have an assert fail somewhere.

When an error eventually occurs, you probably will want to know about it. So
instead of just providing an error message on the command line, we can create a
call to action for people to submit a report.

This should empower people to engage in communication, lowering the chances
people might get frustrated. And making it easier to figure out what might be
causing bugs.

## Usage

```rust no_run
use human_panic_logger::setup_panic_logger;

fn main() {
   setup_panic_logger!("app.log");

   println!("A normal log message");
   panic!("OMG EVERYTHING IS ON FIRE!!!")
}
```
