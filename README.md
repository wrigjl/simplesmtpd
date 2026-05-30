# simplesmtpd

[![CI](https://github.com/wrigjl/simplesmtpd/actions/workflows/ci.yml/badge.svg)](https://github.com/wrigjl/simplesmtpd/actions/workflows/ci.yml)

A simple SMTP server that only parses the stream looking for correct
protocol handling. It accepts any email address or message and throws
them away — it makes no attempt to deliver mail.

What's the point? I wanted an SMTP server my students in CS3337 at ISU
could connect to that I controlled but had no consequences. And, I wanted
an excuse to learn Rust.

## Building

Requires a stable Rust toolchain. Install one via [rustup](https://rustup.rs/) if needed.

```
cargo build --release
```

The binary is at `target/release/simplesmtpd`.

## Running

The server reads from stdin and writes to stdout, designed to be driven by
a systemd socket unit. It listens on port **8025** by default (see the
included socket unit).

The server domain name defaults to `simplesmtp.thought.net` and can be
overridden at startup:

```
simplesmtpd --domain mail.example.com
```

## Deployment (systemd)

Copy the binary and unit files:

```
install -m 755 target/release/simplesmtpd /usr/local/bin/simplesmtpd
cp simplesmtpd.socket simplesmtpd@.service /etc/systemd/system/
systemctl daemon-reload
systemctl enable --now simplesmtpd.socket
```

The socket unit listens on port 8025. Incoming connections are handed off
to `simplesmtpd@.service` via systemd socket activation (`Accept=yes`).

The service runs with filesystem sandboxing enabled (`ProtectSystem=strict`,
`ProtectHome=yes`, `PrivateTmp=yes`, `PrivateDevices=yes`, `NoNewPrivileges=yes`).
The process has no writable filesystem access and cannot escalate privileges.
