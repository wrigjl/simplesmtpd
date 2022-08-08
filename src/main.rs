// Copyright (c) 2022, jason@thought.net
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
    io::{BufRead, BufReader, BufWriter, Error, Write},
    net::{TcpListener, TcpStream},
    thread,
};

enum SmtpState {
    Start,
    Hello,
    Mail,
    Rcpt,
    Data,
    Quit,
}

fn main() -> std::io::Result<()> {
    let (mut tx, rx) = spmc::channel::<TcpStream>();
    let mut handles = Vec::new();

    let listener = TcpListener::bind("0.0.0.0:8025")?;

    for _ in 0..20 {
        let rx = rx.clone();
        handles.push(thread::spawn(move || {
            let msg = rx.recv().unwrap();
            handle_client(&msg).unwrap();
        }));
    }

    for streamres in listener.incoming() {
        match streamres {
            Ok(stream) => tx.send(stream).unwrap(),
            Err(_) => panic!("bad listen"),
        }
    }

    for handle in handles {
        handle.join().unwrap();
    }

    Ok(())
}

fn handle_cmd_mail(
    oldstate: SmtpState,
    writer: &mut BufWriter<&TcpStream>,
) -> Result<SmtpState, Error> {
    match oldstate {
        SmtpState::Hello => {
            writer.write_all("250 ok, let's move on\r\n".as_bytes())?;
            writer.flush()?;
            Ok(SmtpState::Mail)
        }

        _ => {
            writer.write_all("503 bad sequence of commands (didn't say hello)\r\n".as_bytes())?;
            writer.flush()?;
            Ok(oldstate)
        }
    }
}

fn handle_cmd_help(
    oldstate: SmtpState,
    writer: &mut BufWriter<&TcpStream>,
) -> Result<SmtpState, Error> {
    writer.write_all("250 Go read RFC5321.\r\n".as_bytes())?;
    writer.flush()?;
    Ok(oldstate)
}

fn handle_cmd_noop(
    oldstate: SmtpState,
    writer: &mut BufWriter<&TcpStream>,
) -> Result<SmtpState, Error> {
    writer.write_all("250 fine, waste my time.\r\n".as_bytes())?;
    writer.flush()?;
    Ok(oldstate)
}

fn handle_cmd_quit(
    _oldstate: SmtpState,
    writer: &mut BufWriter<&TcpStream>,
) -> Result<SmtpState, Error> {
    writer.write_all("250 yeah, ok, buh bye.\r\n".as_bytes())?;
    writer.flush()?;
    Ok(SmtpState::Quit)
}

fn handle_cmd_rset(
    oldstate: SmtpState,
    writer: &mut BufWriter<&TcpStream>,
) -> Result<SmtpState, Error> {
    writer.write_all("250 reset, fine.\r\n".as_bytes())?;
    writer.flush()?;
    match oldstate {
        SmtpState::Start => Ok(oldstate),
        _ => Ok(SmtpState::Hello),
    }
}

fn handle_cmd_vrfy(
    oldstate: SmtpState,
    writer: &mut BufWriter<&TcpStream>,
) -> Result<SmtpState, Error> {
    writer.write_all("250 yeah, sure, whatever.\r\n".as_bytes())?;
    writer.flush()?;
    Ok(oldstate)
}

fn handle_cmd_rcpt(
    oldstate: SmtpState,
    writer: &mut BufWriter<&TcpStream>,
) -> Result<SmtpState, Error> {
    match oldstate {
        SmtpState::Mail | SmtpState::Rcpt => {
            writer.write_all("250 ok, let's move on\r\n".as_bytes())?;
            writer.flush()?;
            Ok(SmtpState::Rcpt)
        }
        _ => {
            writer.write_all(
                "503 bad sequence of commands (did you say MAIL FROM?)\r\n".as_bytes(),
            )?;
            writer.flush()?;
            Ok(SmtpState::Hello)
        }
    }
}

fn handle_cmd_helo(
    line: &String,
    oldstate: SmtpState,
    writer: &mut BufWriter<&TcpStream>,
) -> Result<SmtpState, Error> {
    let chunks: Vec<_> = line.split(' ').collect();

    if chunks.len() == 1 {
        writer.write_all("501 missing argument\r\n".as_bytes())?;
        writer.flush()?;
        return Ok(oldstate);
    }
    if chunks.len() > 2 {
        writer.write_all("501 too many arguments\r\n".as_bytes())?;
        writer.flush()?;
        return Ok(oldstate);
    }

    //
    // This only verifies that it parses correctly as a domain name
    // this-domain-shouldnt-exists.com would be fine whether or not
    // the domain is actually registered or not.
    //
    match addr::parse_domain_name(chunks[1]) {
        Err(_) => {
            writer.write_all("501 invalid argument (not a valid domain name)\r\n".as_bytes())?;
            writer.flush()?;
            Ok(oldstate)
        }
        Ok(_) => {
            writer.write_all("250 howdy!\r\n".as_bytes())?;
            writer.flush()?;
            Ok(SmtpState::Hello)
        }
    }
}

fn handle_cmd_ehlo(
    _oldstate: SmtpState,
    writer: &mut BufWriter<&TcpStream>,
) -> Result<SmtpState, Error> {
    writer.write_all("250-thought.net greets you.\r\n".as_bytes())?;
    writer.write_all("250 HELP\r\n".as_bytes())?;
    Ok(SmtpState::Hello)
}

fn handle_cmd_data(
    oldstate: SmtpState,
    writer: &mut BufWriter<&TcpStream>,
) -> Result<SmtpState, Error> {
    match oldstate {
        SmtpState::Rcpt => {
            writer.write_all("354 give me the message (. by itself to end)\r\n".as_bytes())?;
            writer.flush()?;
            Ok(SmtpState::Data)
        }
        _ => {
            writer
                .write_all("503 bad sequence of commands (did you say RCPT TO?)\r\n".as_bytes())?;
            writer.flush()?;
            Ok(oldstate)
        }
    }
}

fn handle_cmd_unknown(
    oldstate: SmtpState,
    writer: &mut BufWriter<&TcpStream>,
) -> Result<SmtpState, Error> {
    writer.write_all("502 command not implemented\r\n".as_bytes())?;
    writer.flush()?;
    Ok(oldstate)
}

fn handle_client(stream: &TcpStream) -> Result<(), Error> {
    let mut writer = BufWriter::new(stream);
    let reader = BufReader::new(stream);

    let mut state = SmtpState::Start;

    writer.write_all("220 bogus email service ready\r\n".as_bytes())?;
    writer.flush()?;

    for res in reader.lines() {
        match res {
            Err(_) => {
                println!("failed to read string\n");
                return Ok(());
            }
            Ok(line) => match state {
                SmtpState::Data => {
                    // In data mode, just read a line and see if we're done reading the
                    // message.
                    if line.as_bytes() == ".".as_bytes() {
                        writer.write_all("250 duly noted and ignored, thanks.\r\n".as_bytes())?;
                        writer.flush()?;
                        state = SmtpState::Hello;
                    }
                }
                _ => {
                    // Otherwise, we're in command mode, parse up the verb and branch
                    let command = line.split_ascii_whitespace().next();

                    let cmd = match command {
                        Some(x) => String::from(x).to_ascii_uppercase(),
                        None => String::from(""),
                    };

                    match cmd.as_str() {
                        "DATA" => state = handle_cmd_data(state, &mut writer)?,
                        "EHLO" => state = handle_cmd_ehlo(state, &mut writer)?,
                        "HELO" => state = handle_cmd_helo(&line, state, &mut writer)?,
                        "HELP" => state = handle_cmd_help(state, &mut writer)?,
                        "MAIL" => state = handle_cmd_mail(state, &mut writer)?,
                        "NOOP" => state = handle_cmd_noop(state, &mut writer)?,
                        "QUIT" => state = handle_cmd_quit(state, &mut writer)?,
                        "RCPT" => state = handle_cmd_rcpt(state, &mut writer)?,
                        "RSET" => state = handle_cmd_rset(state, &mut writer)?,
                        "VRFY" => state = handle_cmd_vrfy(state, &mut writer)?,
                        _ => state = handle_cmd_unknown(state, &mut writer)?,
                    }
                }
            },
        }

        if let SmtpState::Quit = state {
            // If state just changed to QUIT, we're done
            break;
        }
    }
    Ok(())
}
