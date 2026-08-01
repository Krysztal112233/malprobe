//! Thin wrapper around `clamav-client` for the scan stage.
//!
//! The upstream crate returns the raw clamd INSTREAM response as `Vec<u8>`
//! and errors are `io::Error`-level, so this module owns the two things the
//! worker actually cares about: a bounded scan duration and a typed verdict.
//!
//! clamd INSTREAM responses are single NUL-terminated lines, either
//! `stream: OK` or `stream: <signature> FOUND` (or `... ERROR`).

use std::path::Path;
use std::time::Duration;

use clamav_client::tokio::{Tcp, scan_file as clamd_scan_file};

/// Verdict of a completed clamd INSTREAM scan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanVerdict {
    Clean,
    /// Malware detected; the value is the signature name from clamd.
    Found(String),
}

/// Streams the file at `path` to clamd and parses the verdict.
///
/// The whole exchange is bounded by `timeout`; a slow clamd (or a huge file)
/// fails the scan instead of hanging the worker forever.
pub async fn scan_path(
    path: &Path,
    clamd_addr: &str,
    timeout: Duration,
) -> Result<ScanVerdict, String> {
    match tokio::time::timeout(
        timeout,
        clamd_scan_file(
            path,
            Tcp {
                host_address: clamd_addr,
            },
            None,
        ),
    )
    .await
    {
        Err(_) => Err(format!("clamd scan timed out after {timeout:?}")),
        Ok(Err(error)) => Err(format!("clamd scan failed: {error}")),
        Ok(Ok(response)) => parse_response(&response),
    }
}

/// Parses a raw clamd INSTREAM response into a [`ScanVerdict`].
///
/// Matches the response strictly so an unexpected payload can never be
/// mistaken for a clean verdict (a scanner must fail closed, not open).
fn parse_response(response: &[u8]) -> Result<ScanVerdict, String> {
    let text = String::from_utf8_lossy(response);
    let line = text.trim_end_matches('\0').trim();
    if line == "stream: OK" {
        Ok(ScanVerdict::Clean)
    } else if let Some(signature) = line.strip_suffix(" FOUND") {
        // `stream: Eicar-Signature FOUND` -> `Eicar-Signature`
        let signature = signature
            .rsplit_once(": ")
            .map(|(_, name)| name)
            .unwrap_or(signature)
            .trim();
        Ok(ScanVerdict::Found(signature.to_owned()))
    } else {
        Err(format!("unexpected clamd response: {line:?}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_clean_response() {
        assert_eq!(parse_response(b"stream: OK\0").unwrap(), ScanVerdict::Clean);
    }

    #[test]
    fn parses_found_response() {
        assert_eq!(
            parse_response(b"stream: Eicar-Test-Signature FOUND\0").unwrap(),
            ScanVerdict::Found("Eicar-Test-Signature".to_owned())
        );
    }

    #[test]
    fn parses_found_response_without_stream_prefix() {
        assert_eq!(
            parse_response(b"Eicar-Test-Signature FOUND\0").unwrap(),
            ScanVerdict::Found("Eicar-Test-Signature".to_owned())
        );
    }

    #[test]
    fn rejects_error_response() {
        let error = parse_response(b"INSTREAM size limit exceeded. ERROR\0").unwrap_err();
        assert!(error.contains("ERROR"));
    }

    #[test]
    fn rejects_ok_suffixed_garbage() {
        // `ends_with("OK")` would classify this as clean; a scanner must
        // never fail open on an unexpected payload.
        assert!(parse_response(b"stream: NOTOK\0").is_err());
        assert!(parse_response(b"stream: OK:NOT\0").is_err());
    }

    #[test]
    fn rejects_empty_response() {
        assert!(parse_response(b"\0").is_err());
    }
}
