// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.

//! The `--aicontrol` JSONL named-pipe client (M9-D1 / Checkpoint #1).
//!
//! Connects to a spawned binary's `.aicontrol` pipe as a client, writes one
//! [`Command`](crate::wire::Command) per line, and reads one
//! [`Reply`](crate::wire::Reply) per line. The server is strictly serial per
//! connection (one in-flight command), so the client is too: [`send`] writes
//! then reads exactly one reply.
//!
//! Windows-only — the binaries' `.aicontrol` server is a Windows named pipe
//! (D-043). A non-Windows build compiles but errors at connect, so the rest of
//! the harness type-checks on any host.
//!
//! [`send`]: AicontrolClient::send

use std::time::Duration;

use crate::wire::{Command, Reply};
use crate::Result;

/// How long [`AicontrolClient::connect`] retries before giving up — the pipe
/// server may not have created its first instance the instant the process is
/// spawned.
pub const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

#[cfg(windows)]
mod imp {
    use super::*;
    use anyhow::{anyhow, Context};
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeClient};
    use tokio::time::Instant;

    // Win32 error codes we retry on while the server comes up.
    const ERROR_FILE_NOT_FOUND: i32 = 2; // pipe instance not yet created
    const ERROR_PIPE_BUSY: i32 = 231; // all instances busy; wait + retry

    /// A connected `.aicontrol` session.
    pub struct AicontrolClient {
        writer: tokio::io::WriteHalf<NamedPipeClient>,
        reader: BufReader<tokio::io::ReadHalf<NamedPipeClient>>,
        pipe: String,
        line: String,
    }

    impl AicontrolClient {
        /// Connect to `pipe`, retrying transient "not yet up"/"busy" errors
        /// until `timeout` elapses.
        pub async fn connect(pipe: &str, timeout: Duration) -> Result<AicontrolClient> {
            let deadline = Instant::now() + timeout;
            loop {
                match ClientOptions::new().open(pipe) {
                    Ok(client) => {
                        let (r, w) = tokio::io::split(client);
                        return Ok(AicontrolClient {
                            writer: w,
                            reader: BufReader::new(r),
                            pipe: pipe.to_string(),
                            line: String::new(),
                        });
                    }
                    Err(e)
                        if matches!(
                            e.raw_os_error(),
                            Some(ERROR_FILE_NOT_FOUND) | Some(ERROR_PIPE_BUSY)
                        ) =>
                    {
                        if Instant::now() >= deadline {
                            return Err(anyhow!(
                                "timed out connecting to aicontrol pipe {pipe} after {timeout:?}: {e}"
                            ));
                        }
                        tokio::time::sleep(Duration::from_millis(50)).await;
                    }
                    Err(e) => {
                        return Err(anyhow!("opening aicontrol pipe {pipe}: {e}"));
                    }
                }
            }
        }

        /// Send one command and read exactly one reply.
        pub async fn send(&mut self, cmd: &Command) -> Result<Reply> {
            self.send_line(&cmd.to_line()).await
        }

        /// Send a pre-serialized JSONL command line (the batch runner sends the
        /// scenario author's exact JSON after `{{key}}` substitution, so it is
        /// not re-parsed/re-serialized through [`Command`]) and read one reply.
        pub async fn send_line(&mut self, json_line: &str) -> Result<Reply> {
            let mut out = json_line.trim_end().to_string();
            out.push('\n');
            self.writer
                .write_all(out.as_bytes())
                .await
                .with_context(|| format!("writing command to {}", self.pipe))?;
            self.writer
                .flush()
                .await
                .with_context(|| format!("flushing command to {}", self.pipe))?;

            self.line.clear();
            let n = self
                .reader
                .read_line(&mut self.line)
                .await
                .with_context(|| format!("reading reply from {}", self.pipe))?;
            if n == 0 {
                return Err(anyhow!(
                    "aicontrol pipe {} closed before replying to line `{}`",
                    self.pipe,
                    json_line.trim_end()
                ));
            }
            let trimmed = self.line.trim_end_matches('\n').trim_end_matches('\r');
            Reply::from_line(trimmed)
                .with_context(|| format!("parsing reply from {}: {trimmed}", self.pipe))
        }

        /// Convenience: send a bare verb (no args).
        pub async fn send_verb(&mut self, verb: &str) -> Result<Reply> {
            self.send(&Command::new(verb)).await
        }
    }
}

#[cfg(not(windows))]
mod imp {
    use super::*;

    /// Non-Windows stub — the `.aicontrol` server is a Windows named pipe.
    pub struct AicontrolClient;

    impl AicontrolClient {
        pub async fn connect(pipe: &str, _timeout: Duration) -> Result<AicontrolClient> {
            Err(anyhow::anyhow!(
                "aicontrol named-pipe client is Windows-only (pipe {pipe})"
            ))
        }
        pub async fn send(&mut self, _cmd: &Command) -> Result<Reply> {
            Err(anyhow::anyhow!("aicontrol client is Windows-only"))
        }
        pub async fn send_line(&mut self, _json_line: &str) -> Result<Reply> {
            Err(anyhow::anyhow!("aicontrol client is Windows-only"))
        }
        pub async fn send_verb(&mut self, _verb: &str) -> Result<Reply> {
            Err(anyhow::anyhow!("aicontrol client is Windows-only"))
        }
    }
}

pub use imp::AicontrolClient;
