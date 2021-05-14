//! Exposes the JavaScript event loop for scheduling asynchronous events.
//!
//! The [_event loop_][event-loop] is how Node.js provides JavaScript programs
//! access to concurrent events such as the completion of [file][fs] or
//! [network][net] operations, scheduled [timers][timer], or receiving messages
//! from other [processes][process].
//!
//! When an asynchronous operation is started from JavaScript, it registers
//! a JavaScript callback function to wait for the operation to complete. When
//! the operation completes, the callback and the result data are added to an
//! internal _event queue_ in the Node.js runtime so that the event can be
//! processed in order.
//!
//! The event loop processes completed events one at a time in the JavaScript
//! execution thread by calling the registered callback function with its result
//! value as an argument.
//!
//! ## Creating Custom Events
//!
//! This module allows Neon programs to create new types of concurrent events
//! in Rust and expose them to JavaScript as asynchronous functions.
//!
//! A common use for custom events is to run expensive or long-lived
//! computations in a background thread without blocking the JavaScript
//! thread. For example, using the [`psd` crate][psd-crate], a Neon program could
//! asynchronously parse (potentially large) [PSD files][psd-file] in a
//! background thread:
//!
//! ```
//! # #[cfg(feature = "napi-1")] {
//! # use neon::prelude::*;
//! #
//! # fn parse(filename: String, callback: Root<JsFunction>, queue: EventQueue) { }
//! #
//! fn parse_async(mut cx: FunctionContext) -> JsResult<JsUndefined> {
//!     // The types `String`, `Root<JsFunction>`, and `EventQueue` can all be
//!     // sent across threads.
//!     let filename = cx.argument::<JsString>(0)?.value(&mut cx);
//!     let callback = cx.argument::<JsFunction>(1)?.root(&mut cx);
//!     let queue = cx.queue();
//!
//!     // Spawn a thread to complete the execution. This will _not_ block the
//!     // JavaScript event loop.
//!     std::thread::spawn(move || {
//!         // Do the heavy lifting inside the background thread.
//!         parse(filename, callback, queue);
//!     });
//!
//!     Ok(cx.undefined())
//! }
//! # }
//! ```
//!
//! (Note that this usage of [`spawn`](std::thread::spawn) makes use of Rust's
//! [`move`][move] syntax to transfer ownership of data to the background
//! thread.)
//!
//! Upon completion of its task, the background thread can use the JavaScript
//! callback and the event queue to notify the main thread of the result:
//!
//! ```
//! # extern crate psd;
//! # extern crate failure;
//! #
//! # #[cfg(feature = "napi-1")] fn main() {
//! # use neon::prelude::*;
//! # use psd::Psd;
//! # use failure::Error;
//! #
//! fn psd_from_filename(filename: String) -> Result<Psd, Error> {
//!     Psd::from_bytes(&std::fs::read(&filename)?)
//! }
//!
//! fn parse(filename: String, callback: Root<JsFunction>, queue: EventQueue) {
//!     let result = psd_from_filename(filename);
//!
//!     // Send a closure as a task to be executed by the JavaScript event
//!     // queue. This _will_ block the event queue while executing.
//!     queue.send(move |mut cx| {
//!         let callback = callback.into_inner(&mut cx);
//!         let this = cx.undefined();
//!         let null = cx.null();
//!         let args = match result {
//!             Ok(psd) => {
//!                 // Extract data from the parsed file.
//!                 let width = cx.number(psd.width());
//!                 let height = cx.number(psd.height());
//!
//!                 // Save the data in a result object.
//!                 let obj = cx.empty_object();
//!                 obj.set(&mut cx, "width", width)?;
//!                 obj.set(&mut cx, "height", height)?;
//!                 vec![
//!                     cx.null().upcast::<JsValue>(),
//!                     obj.upcast(),
//!                 ]
//!             }
//!             Err(_) => {
//!                 // A full implementation would provide
//!                 // more detailed error diagnostics.
//!                 let err = cx.string("PSD parse error");
//!                 vec![
//!                     err.upcast::<JsValue>(),
//!                 ]
//!             }
//!         };
//!
//!         callback.call(&mut cx, this, args)?;
//!
//!         Ok(())
//!     });
//! }
//! # }
//! ```
//!
//! ## See also
//!
//! 1. Panu Pitkamaki. [Event loop from 10,000ft][event-loop].
//!
//! [event-loop]: https://bytearcher.com/articles/event-loop-10-000ft/
//! [fs]: https://nodejs.org/dist/latest/docs/api/fs.html
//! [net]: https://nodejs.org/dist/latest/docs/api/net.html
//! [process]: https://nodejs.org/dist/latest/docs/api/process.html
//! [timer]: https://nodejs.org/dist/latest/docs/api/timers.html
//! [move]: https://doc.rust-lang.org/std/keyword.move.html
//! [psd-crate]: https://crates.io/crates/psd
//! [psd-file]: https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/

#[cfg(all(feature = "napi-4", feature = "event-queue-api"))]
mod event_queue;

#[cfg(all(feature = "napi-4", feature = "event-queue-api"))]
pub use self::event_queue::{EventQueue, EventQueueError};

#[cfg(all(not(feature = "napi-1"), feature = "event-handler-api"))]
mod event_handler;

#[cfg(all(not(feature = "napi-1"), feature = "event-handler-api"))]
pub use self::event_handler::EventHandler;

#[cfg(all(feature = "napi-1", feature = "event-handler-api"))]
compile_error!(
    "The `EventHandler` API is not supported with the N-API \
    backend. Use `EventQueue` instead."
);
