// Copyright (c) 2022, Jason L. Wright <jason@thought.net>
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
//
// 1. Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
// 2. Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in
//    the documentation and/or other materials provided with the
//    distribution.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
//

//
// This is a really simple SMTP service. It doesn't do anything more
// than parse the commands and return success if the protocol appears
// to be followed correctly. This is mainly so that my students in
// CS3337 at ISU have an SMTP server to play with without consequences.
//
// Also, it is an excuse for me to play with rust development.
//

use std::{
    io::{BufRead, BufReader, BufWriter, Error, Read, Write},
    str::FromStr,
};


pub enum SmtpState {
    Start,
    Hello,
    Mail,
    Rcpt,
    Data,
    Quit,
}

pub fn handle_cmd_mail(
    line: &str,
    oldstate: SmtpState,
    mut writer: impl Write,
) -> Result<SmtpState, Error> {
    if !matches!(oldstate, SmtpState::Hello) {
        writer.write_all("503 bad sequence of commands -- EHLO first, this is not optional\r\n".as_bytes())?;
        return Ok(oldstate);
    }

    if !line.to_ascii_uppercase().starts_with("MAIL FROM:") {
        writer.write_all("501 try MAIL FROM:<address>, with the angle brackets and everything\r\n".as_bytes())?;
        return Ok(oldstate);
    }

    let arg = &line["MAIL FROM:".len()..];
    if arg.is_empty() || !arg.starts_with('<') || !arg.ends_with('>') {
        writer.write_all("501 angle brackets are not optional: MAIL FROM:<address> or MAIL FROM:<>\r\n".as_bytes())?;
        return Ok(oldstate);
    }

    writer.write_all("250 sure, I'll pretend that's a real address\r\n".as_bytes())?;
    Ok(SmtpState::Mail)
}

pub fn handle_cmd_help(
    oldstate: SmtpState,
    mut writer: impl Write,
) -> Result<SmtpState, Error> {
    writer.write_all("214 Go read RFC5321.\r\n".as_bytes())?;
    Ok(oldstate)
}

pub fn handle_cmd_noop(
    oldstate: SmtpState,
    mut writer: impl Write,
) -> Result<SmtpState, Error> {
    writer.write_all("250 congratulations on accomplishing absolutely nothing\r\n".as_bytes())?;
    Ok(oldstate)
}

pub fn handle_cmd_quit(
    line: &str,
    oldstate: SmtpState,
    mut writer: impl Write,
    domain: &str,
) -> Result<SmtpState, Error> {
    if line.split_ascii_whitespace().count() > 1 {
        writer.write_all("501 QUIT takes no arguments -- it's literally one word\r\n".as_bytes())?;
        return Ok(oldstate);
    }
    writer.write_all(format!("221 {} fine, goodbye, don't let the door hit you\r\n", domain).as_bytes())?;
    Ok(SmtpState::Quit)
}

pub fn handle_cmd_rset(
    line: &str,
    oldstate: SmtpState,
    mut writer: impl Write,
) -> Result<SmtpState, Error> {
    if line.split_ascii_whitespace().count() > 1 {
        writer.write_all("501 RSET takes no arguments, what are you even doing\r\n".as_bytes())?;
        return Ok(oldstate);
    }
    writer.write_all("250 fine, we can start this disaster over\r\n".as_bytes())?;
    match oldstate {
        SmtpState::Start => Ok(oldstate),
        _ => Ok(SmtpState::Hello),
    }
}

pub fn handle_cmd_vrfy(
    line: &str,
    oldstate: SmtpState,
    mut writer: impl Write,
) -> Result<SmtpState, Error> {
    if !line.to_ascii_uppercase().starts_with("VRFY ") {
        writer.write_all("501 VRFY requires an argument -- who exactly are you looking for?\r\n".as_bytes())?;
        return Ok(oldstate);
    }

    let arg = &line["VRFY ".len()..];
    if arg.is_empty() {
        writer.write_all("501 VRFY requires an argument -- who exactly are you looking for?\r\n".as_bytes())?;
        return Ok(oldstate);
    }

    writer.write_all("252 couldn't tell you, wouldn't tell you, but sure, send mail\r\n".as_bytes())?;
    Ok(oldstate)
}

pub fn handle_cmd_expn(
    line: &str,
    oldstate: SmtpState,
    mut writer: impl Write,
) -> Result<SmtpState, Error> {
    if !line.to_ascii_uppercase().starts_with("EXPN ") {
        writer.write_all("501 EXPN requires a list name -- I'm not a mind reader\r\n".as_bytes())?;
        return Ok(oldstate);
    }

    let arg = &line["EXPN ".len()..];
    if arg.is_empty() {
        writer.write_all("501 EXPN requires a list name -- I'm not a mind reader\r\n".as_bytes())?;
        return Ok(oldstate);
    }

    writer.write_all("252 I'm not your mailing list directory\r\n".as_bytes())?;
    Ok(oldstate)
}

pub fn handle_cmd_rcpt(
    line: &str,
    oldstate: SmtpState,
    mut writer: impl Write,
) -> Result<SmtpState, Error> {
    if !matches!(oldstate, SmtpState::Rcpt) && !matches!(oldstate, SmtpState::Mail) {
        writer.write_all("503 bad sequence of commands -- MAIL FROM first, that's how this works\r\n".as_bytes())?;
        return Ok(oldstate);
    }

    if !line.to_ascii_uppercase().starts_with("RCPT TO:") {
        writer.write_all("501 try RCPT TO:<address>, the angle brackets are load-bearing\r\n".as_bytes())?;
        return Ok(oldstate);
    }

    let arg = &line["RCPT TO:".len()..];
    if arg.is_empty() || !arg.starts_with('<') || !arg.ends_with('>') {
        writer.write_all("501 the angle brackets are load-bearing: RCPT TO:<address>\r\n".as_bytes())?;
        return Ok(oldstate);
    }

    writer.write_all("250 another recipient, sure, the more the merrier I guess\r\n".as_bytes())?;
    Ok(SmtpState::Rcpt)
}

pub fn handle_cmd_helo(
    line: &str,
    oldstate: SmtpState,
    mut writer: impl Write,
    domain: &str,
) -> Result<SmtpState, Error> {
    let chunks: Vec<_> = line.split(' ').collect();

    if chunks.len() == 1 {
        writer.write_all("501 HELO requires a domain argument -- who are you exactly?\r\n".as_bytes())?;
        return Ok(oldstate);
    }
    if chunks.len() > 2 {
        writer.write_all("501 one domain will do, settle down\r\n".as_bytes())?;
        return Ok(oldstate);
    }

    match addr::parse_domain_name(chunks[1]) {
        Err(_) => {
            writer.write_all("501 that's not a valid domain name, nice try\r\n".as_bytes())?;
            Ok(oldstate)
        }
        Ok(_) => {
            writer.write_all(format!("250 {} howdy, I guess\r\n", domain).as_bytes())?;
            Ok(SmtpState::Hello)
        }
    }
}

fn cmd_ehlo_response(mut writer: impl Write, domain: &str) -> Result<SmtpState, Error> {
    writer.write_all(format!("250-{} oh, it's you\r\n", domain).as_bytes())?;
    writer.write_all("250-EXPN\r\n".as_bytes())?;
    writer.write_all("250 HELP\r\n".as_bytes())?;
    Ok(SmtpState::Hello)
}

pub fn handle_cmd_ehlo(
    line: &str,
    oldstate: SmtpState,
    mut writer: impl Write,
    domain: &str,
) -> Result<SmtpState, Error> {
    let chunks: Vec<_> = line.split(' ').collect();

    if chunks.len() == 1 {
        writer.write_all("501 EHLO requires a domain argument -- who are you exactly?\r\n".as_bytes())?;
        return Ok(oldstate);
    }
    if chunks.len() > 2 {
        writer.write_all("501 one domain will do, settle down\r\n".as_bytes())?;
        return Ok(oldstate);
    }

    let arg = chunks[1];

    if arg.is_empty() {
        writer.write_all("501 a zero-length domain name, very creative\r\n".as_bytes())?;
        return Ok(oldstate);
    }

    // RFC 5321 §4.1.3: argument is one of:
    //   "[ipv4addr]"        IPv4 address literal
    //   "[IPv6:ipv6addr]"   IPv6 address literal
    //   "[tag:value]"       general address literal (accepted as-is)
    //   "domain.example"    domain name
    if arg.starts_with('[') && arg.ends_with(']') {
        let inner = &arg[1..arg.len() - 1];

        if std::net::IpAddr::from_str(inner).is_ok() {
            return cmd_ehlo_response(writer, domain);
        }

        if let Some(addr) = inner.strip_prefix("IPv6:") {
            if std::net::Ipv6Addr::from_str(addr).is_err() {
                writer.write_all("501 that's not a valid IPv6 address, nice try\r\n".as_bytes())?;
                return Ok(oldstate);
            }
            return cmd_ehlo_response(writer, domain);
        }

        // General address literal — accept without further validation.
    } else if addr::parse_domain_name(arg).is_err() {
        writer.write_all("501 that's not a valid domain name, nice try\r\n".as_bytes())?;
        return Ok(oldstate);
    }

    cmd_ehlo_response(writer, domain)
}

pub fn handle_cmd_data(
    line: &str,
    oldstate: SmtpState,
    mut writer: impl Write,
) -> Result<SmtpState, Error> {
    if line.split_ascii_whitespace().count() > 1 {
        writer.write_all("501 DATA takes no arguments, are you okay?\r\n".as_bytes())?;
        return Ok(oldstate);
    }
    match oldstate {
        SmtpState::Rcpt => {
            writer.write_all("354 go ahead, I'll ignore it carefully; end with <CRLF>.<CRLF>\r\n".as_bytes())?;
            Ok(SmtpState::Data)
        }
        _ => {
            writer.write_all("503 bad sequence of commands -- who exactly is this mail for?\r\n".as_bytes())?;
            Ok(oldstate)
        }
    }
}

fn handle_cmd_unknown(
    oldstate: SmtpState,
    mut writer: impl Write,
) -> Result<SmtpState, Error> {
    writer.write_all("500 command unrecognized -- did you even read the RFC?\r\n".as_bytes())?;
    Ok(oldstate)
}

pub fn handle_client(fin: impl Read, fout: impl Write, domain: &str) -> Result<(), Error> {
    let mut writer = BufWriter::new(fout);
    let reader = BufReader::new(fin);

    let mut state = SmtpState::Start;

    writer.write_all(format!("220 {} welcome, I guess\r\n", domain).as_bytes())?;
    writer.flush()?;

    for res in reader.lines() {
        match res {
            Err(_) => {
                return Ok(());
            }
            Ok(line) => match state {
                SmtpState::Data => {
                    // In data mode, consume lines until the end-of-data indicator (RFC 5321 §4.5.2).
                    if line == "." {
                        writer.write_all("250 duly noted and promptly ignored, thanks\r\n".as_bytes())?;
                        state = SmtpState::Hello;
                    }
                }
                _ => {
                    let cmd = line
                        .split_ascii_whitespace()
                        .next()
                        .map(|x| x.to_ascii_uppercase())
                        .unwrap_or_default();

                    match cmd.as_str() {
                        "DATA" => state = handle_cmd_data(&line, state, &mut writer)?,
                        "EHLO" => state = handle_cmd_ehlo(&line, state, &mut writer, domain)?,
                        "EXPN" => state = handle_cmd_expn(&line, state, &mut writer)?,
                        "HELO" => state = handle_cmd_helo(&line, state, &mut writer, domain)?,
                        "HELP" => state = handle_cmd_help(state, &mut writer)?,
                        "MAIL" => state = handle_cmd_mail(&line, state, &mut writer)?,
                        "NOOP" => state = handle_cmd_noop(state, &mut writer)?,
                        "QUIT" => state = handle_cmd_quit(&line, state, &mut writer, domain)?,
                        "RCPT" => state = handle_cmd_rcpt(&line, state, &mut writer)?,
                        "RSET" => state = handle_cmd_rset(&line, state, &mut writer)?,
                        "VRFY" => state = handle_cmd_vrfy(&line, state, &mut writer)?,
                        _ => state = handle_cmd_unknown(state, &mut writer)?,
                    }
                }
            },
        }

        if let SmtpState::Quit = state {
            break;
        }
        writer.flush()?;
    }
    Ok(())
}
