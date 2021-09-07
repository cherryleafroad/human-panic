//! Panic messages for humans
//!
//! Handles panics by calling
//! [`std::panic::set_hook`](https://doc.rust-lang.org/std/panic/fn.set_hook.html)
//! to make errors nice for humans.
//!
//! ## Why?
//! When you're building a CLI, polish is super important. Even though Rust is
//! pretty great at safety, it's not unheard of to access the wrong index in a
//! vector or have an assert fail somewhere.
//!
//! When an error eventually occurs, you probably will want to know about it. So
//! instead of just providing an error message on the command line, we can create a
//! call to action for people to submit a report.
//!
//! This should empower people to engage in communication, lowering the chances
//! people might get frustrated. And making it easier to figure out what might be
//! causing bugs.
//!
//! ### Default Output
//!
//! ```txt
//! thread 'main' panicked at 'oops', examples/main.rs:2:3
//! note: Run with `RUST_BACKTRACE=1` for a backtrace.
//! ```
//!
//! ### Human-Panic Output
//!
//! ```txt
//! Well, this is embarrassing.
//!
//! human-panic had a problem and crashed. To help us diagnose the problem you can send us a crash report.
//!
//! We have generated a report file at "/var/folders/zw/bpfvmq390lv2c6gn_6byyv0w0000gn/T/report-8351cad6-d2b5-4fe8-accd-1fcbf4538792.toml". Submit an issue or email with the subject of "human-panic Crash Report" and include the report as an attachment.
//!
//! - Homepage: https://github.com/yoshuawuyts/human-panic
//! - Authors: Yoshua Wuyts <yoshuawuyts@gmail.com>
//!
//! We take privacy seriously, and do not perform any automated error collection. In order to improve the software, we rely on people to submit reports.
//!
//! Thank you kindly!

#![cfg_attr(feature = "nightly", deny(missing_docs))]
#![cfg_attr(feature = "nightly", feature(external_doc))]
#![cfg_attr(feature = "nightly", feature(panic_info_message))]

use std::borrow::Cow;
use std::io::{Result as IoResult, Write};
use std::panic::PanicInfo;
use std::path::{Path, PathBuf};
use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};
use backtrace::Backtrace;
use core::mem;
use std::fmt::Write as WriteFmt;

/// A convenient metadata struct that describes a crate
pub struct Metadata {
    /// The crate version
    pub version: Cow<'static, str>,
    /// The crate name
    pub name: Cow<'static, str>,
    /// The list of authors of the crate
    pub authors: Cow<'static, str>,
    /// The URL of the crate's website
    pub homepage: Cow<'static, str>,
}

/// If in debug mode, sends first param to function. If in release mode, sends 2nd param to function
#[macro_export]
macro_rules! debug_param {
    ($a:expr, $b:expr) => {
        if cfg!(debug_assertions) {
            $a
        } else {
            $b
        }
    };
}

/// `human-panic` initialisation macro
///
/// You can either call this macro with no arguments `setup_panic!()` or
/// with a Metadata struct, if you don't want the error message to display
/// the values used in your `Cargo.toml` file.
///
/// The Metadata struct can't implement `Default` because of orphan rules, which
/// means you need to provide all fields for initialisation.
///
/// ```
/// use human_panic::setup_human_panic_logger;
///
/// setup_human_panic_logger!(Metadata {
///     name: env!("CARGO_PKG_NAME").into(),
///     version: env!("CARGO_PKG_VERSION").into(),
///     authors: "My Company Support <support@mycompany.com>".into(),
///     homepage: "support.mycompany.com".into(),
/// });
/// ```
#[macro_export]
macro_rules! setup_human_panic_logger {
  ($log_file:expr) => {
    use std::panic::{self, PanicInfo};
    use $crate::{format_panic, print_msg, Metadata, debug_param};
    use simplelog::*;

    CombinedLogger::init(
        vec![
            WriteLogger::new(
                debug_param!(LevelFilter::Debug, LevelFilter::Info),
                Config::default(),
                OpenOptions::new()
                    .read(true)
                    .append(true)
                    .create(true)
                    .open(&$log_file).unwrap()),
        ]
    ).unwrap();

    let meta = human_panic_logger::Metadata {
        version: env!("CARGO_PKG_VERSION").into(),
        name: env!("CARGO_PKG_NAME").into(),
        authors: env!("CARGO_PKG_AUTHORS").replace(":", ", ").into(),
        homepage: env!("CARGO_PKG_HOMEPAGE").into(),
    };

    let default_hook = panic::take_hook();

    if let Err(_) = ::std::env::var("RUST_BACKTRACE") {
        panic::set_hook(Box::new(move |info: &panic::PanicInfo| {
            // call standard hook in debug mode
            #[cfg(debug_assertions)]
            default_hook(info);

            // output panic to logfile
            error!("Panic! :: {}", format_panic(info));

            // do human error message in release mode
            #[cfg(not(debug_assertions))]
            human_panic::print_msg($log, &meta)
                .expect("human-panic-logger: printing error message to console failed");
        }));
    }
  };
}

/// Utility function that prints a message to our human users
pub fn print_msg<P: AsRef<Path>>(
    file_path: P,
    meta: &Metadata,
) -> IoResult<()> {
    let (_version, name, authors, homepage) =
        (&meta.version, &meta.name, &meta.authors, &meta.homepage);

    let stderr = BufferWriter::stderr(ColorChoice::Auto);
    let mut buffer = stderr.buffer();
    buffer.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;

    writeln!(&mut buffer, "Well, this is embarrassing.\n")?;
    writeln!(
        &mut buffer,
        "{} had a problem and crashed. To help us diagnose the \
     problem you can send us a crash report.\n",
        name
    )?;
    writeln!(
        &mut buffer,
        "There is a log file of the crash at \"{}\". Please submit an \
     issue or email with the subject of \"{} Crash Report\" and include the \
     log as an attachment.\n",
        fp.as_ref().display(),
        name
    )?;

    if !homepage.is_empty() {
        writeln!(&mut buffer, "- Homepage: {}", homepage)?;
    }
    if !authors.is_empty() {
        writeln!(&mut buffer, "- Authors: {}", authors)?;
    }
    writeln!(
        &mut buffer,
        "\nWe take privacy seriously, and do not perform any \
     automated error collection. In order to improve the software, we rely on \
     people to submit reports.\n"
    )?;
    writeln!(&mut buffer, "Thank you!")?;

    buffer.reset()?;

    stderr.print(&buffer).unwrap();
    Ok(())
}

/// Format the panic message for printing to log
pub fn format_panic(panic_info: &PanicInfo) -> String {
    let mut expl = String::new();

    #[cfg(feature = "nightly")]
        let message = panic_info.message().map(|m| format!("{}", m));

    #[cfg(not(feature = "nightly"))]
        let message = match (
        panic_info.payload().downcast_ref::<&str>(),
        panic_info.payload().downcast_ref::<String>(),
    ) {
        (Some(s), _) => Some(s.to_string()),
        (_, Some(s)) => Some(s.to_string()),
        (None, None) => None,
    };

    let cause = match message {
        Some(m) => m,
        None => "Unknown".into(),
    };

    match panic_info.location() {
        Some(location) => expl.push_str(&format!(
            "Panic occurred in file '{}' at line {}\n",
            location.file(),
            location.line()
        )),
        None => expl.push_str("Panic location unknown.\n"),
    }

    format!("{}\n   {}\n{}", expl, cause, format_backtrace())
}

fn format_backtrace() -> String {
    //We skip 3 frames from backtrace library
    //Then we skip 3 frames for our own library
    //(including closure that we set as hook)
    //Then we skip 2 functions from Rust's runtime
    //that calls panic hook
    const SKIP_FRAMES_NUM: usize = 8;
    //We take padding for address and extra two letters
    //to padd after index.
    const HEX_WIDTH: usize = mem::size_of::<usize>() + 2;
    //Padding for next lines after frame's address
    const NEXT_SYMBOL_PADDING: usize = HEX_WIDTH + 6;

    let mut backtrace = String::new();

    //Here we iterate over backtrace frames
    //(each corresponds to function's stack)
    //We need to print its address
    //and symbol(e.g. function name),
    //if it is available
    for (idx, frame) in Backtrace::new()
        .frames()
        .iter()
        .skip(SKIP_FRAMES_NUM)
        .enumerate()
    {
        let ip = frame.ip();
        let _ = write!(backtrace, "\n{:4}: {:2$?}", idx, ip, HEX_WIDTH);

        let symbols = frame.symbols();
        if symbols.is_empty() {
            let _ = write!(backtrace, " - <unresolved>");
            continue;
        }

        for (idx, symbol) in symbols.iter().enumerate() {
            //Print symbols from this address,
            //if there are several addresses
            //we need to put it on next line
            if idx != 0 {
                let _ = write!(backtrace, "\n{:1$}", "", NEXT_SYMBOL_PADDING);
            }

            if let Some(name) = symbol.name() {
                let _ = write!(backtrace, " - {}", name);
            } else {
                let _ = write!(backtrace, " - <unknown>");
            }

            //See if there is debug information with file name and line
            if let (Some(file), Some(line)) = (symbol.filename(), symbol.lineno()) {
                let _ = write!(
                    backtrace,
                    "\n{:3$}at {}:{}",
                    "",
                    file.display(),
                    line,
                    NEXT_SYMBOL_PADDING
                );
            }
        }
    }

    backtrace
}
