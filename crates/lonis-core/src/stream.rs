// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Stream mode (ADR-0009): incremental block delivery.
//!
//! `BlockStream` is a synchronous **pull** primitive — an iterator of
//! `Result<Block<P>, ToolError>`. Sync is deliberate: the tool boundary is
//! bytes on a pipe (synchronous by physics), `Box<dyn Tool<P>>` object
//! safety forbids `async fn` in the trait, and a runtime in the library
//! would capture every vertical. Async hosts (Wallace-class session
//! orchestrators) bridge via [`BlockStream::into_async`] (behind the
//! `futures` feature, runtime-agnostic); the async nature of a session
//! lives above the seam, not in it.
//!
//! Backpressure is real: subprocess streaming flows through a bounded
//! channel, so a slow consumer throttles the pipe (and thereby the child).

use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use lonis_schema::{Block, BlockPayload, ToolError};

/// Buffer depth of a subprocess-backed stream. A full buffer means the
/// child's pipe backs up — backpressure by construction.
pub(crate) const STREAM_BUFFER: usize = 64;

/// Kills the child if the stream is dropped early (bounded doctrine: an
/// abandoned stream must not leave a running process).
pub(crate) struct ChildGuard(pub(crate) Arc<Mutex<std::process::Child>>);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        if let Ok(mut child) = self.0.lock() {
            let _ = child.kill();
        }
    }
}

enum Inner<P: BlockPayload> {
    /// A completed `Vec<Block>` replayed as a stream (the default path).
    Collected(std::vec::IntoIter<Result<Block<P>, ToolError>>),
    /// A live channel from a producing child (subprocess streaming). The
    /// guard is never read — it exists to kill the child on early drop.
    Channel {
        rx: mpsc::Receiver<Result<Block<P>, ToolError>>,
        _guard: ChildGuard,
    },
}

/// A pull-based stream of blocks (or a terminal error) from one invocation.
///
/// The iterator ends when the invocation's output is exhausted; a failed
/// invocation yields its [`ToolError`] as the final item (blocks emitted
/// before the failure are delivered first).
pub struct BlockStream<P: BlockPayload> {
    inner: Inner<P>,
}

impl<P: BlockPayload> BlockStream<P> {
    /// A stream over an already-complete block vector (the
    /// `Tool::invoke_stream` default: collect, then stream).
    #[must_use]
    pub fn from_blocks(blocks: Vec<Block<P>>) -> Self {
        Self {
            inner: Inner::Collected(blocks.into_iter().map(Ok).collect::<Vec<_>>().into_iter()),
        }
    }

    /// A stream backed by a live channel from a producing child.
    pub(crate) fn from_channel(
        rx: mpsc::Receiver<Result<Block<P>, ToolError>>,
        guard: ChildGuard,
    ) -> Self {
        Self {
            inner: Inner::Channel { rx, _guard: guard },
        }
    }

    /// Bridge into an async host: a runtime-agnostic
    /// `futures_core::Stream` (a bridge thread pulls the sync iterator and
    /// forwards; backpressure is handled upstream by the bounded channel).
    ///
    /// Available behind the `futures` feature. The library never names a
    /// runtime — `tokio` is the host's choice, not the contract's.
    #[cfg(feature = "futures")]
    #[must_use]
    pub fn into_async(
        self,
    ) -> futures_channel::mpsc::UnboundedReceiver<Result<Block<P>, ToolError>> {
        let (tx, rx) = futures_channel::mpsc::unbounded();
        std::thread::spawn(move || {
            for item in self {
                if tx.unbounded_send(item).is_err() {
                    break; // async consumer dropped
                }
            }
        });
        rx
    }
}

impl<P: BlockPayload> Iterator for BlockStream<P> {
    type Item = Result<Block<P>, ToolError>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.inner {
            Inner::Collected(iter) => iter.next(),
            Inner::Channel { rx, .. } => rx.recv().ok(),
        }
    }
}
