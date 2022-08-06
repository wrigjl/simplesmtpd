use std::{
    io::Write,
    net::{TcpListener, TcpStream},
};

use spmc;
use std::io::BufRead;
use std::io::Error;
use std::thread;

enum SmtpState {
    Start,
    Hello,
    Mail,
    Rcpt,
    Data,
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

fn handle_client(mut stream: &TcpStream) -> Result<(), Error> {
    let mut writer = std::io::BufWriter::new(stream);
    let reader = std::io::BufReader::new(stream);

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
                    if line.as_bytes() == ".".as_bytes() {
                        writer.write_all("250 duly noted and ignored, thanks.\r\n".as_bytes())?;
                        writer.flush()?;
                        state = SmtpState::Hello;
                    }
                }
                _ => {
                    let command = line.split_ascii_whitespace().next();

                    let cmd = match command {
                        Some(x) => String::from(x).to_ascii_uppercase(),
                        None => String::from(""),
                    };

                    match cmd.as_str() {
                        "HELP" => {
                            writer.write_all("250 Go read RFC822\r\n".as_bytes())?;
                            writer.flush()?;
                        }
                        "NOOP" => {
                            writer.write_all("250 fine, waste my time.\r\n".as_bytes())?;
                            writer.flush()?;
                        }
                        "QUIT" => {
                            writer.write_all("221 yeah, whatever, buh bye.\r\n".as_bytes())?;
                            writer.flush()?;
                            return Ok(());
                        }
                        "VRFY" => {
                            writer.write_all("250 yeah, sure, whatever.\r\n".as_bytes())?;
                            writer.flush()?;
                        }

                        "RSET" => {
                            stream.write_all("250 reset, fine.\r\n".as_bytes())?;
                            state = match state {
                                SmtpState::Start => SmtpState::Start,
                                other => other,
                            };
                        }

                        "HELO" => {
                            state = SmtpState::Hello;
                            writer.write_all("250 howdy!\r\n".as_bytes())?;
                            writer.flush()?;
                        }

                        "EHLO" => {
                            state = SmtpState::Hello;
                            writer.write_all("250-thought.net greets you.\r\n".as_bytes())?;
                            writer.write_all("250 HELP\r\n".as_bytes())?;
                            writer.flush()?;
                        }

                        "MAIL" => {
                            state = match state {
                                SmtpState::Hello => {
                                    writer.write_all("250 ok, let's move on\r\n".as_bytes())?;
                                    writer.flush()?;
                                    SmtpState::Mail
                                }
                                _ => {
                                    writer.write_all(
                                        "503 bad sequence of commands (didn't say hello)\r\n"
                                            .as_bytes(),
                                    )?;
                                    writer.flush()?;
                                    SmtpState::Hello
                                }
                            };
                        }

                        "RCPT" => {
                            state = match state {
                                SmtpState::Mail => {
                                    writer.write_all("250 ok, let's move on\r\n".as_bytes())?;
                                    writer.flush()?;
                                    SmtpState::Rcpt
                                }
                                SmtpState::Rcpt => {
                                    writer.write_all("250 ok, let's move on\r\n".as_bytes())?;
                                    writer.flush()?;
                                    SmtpState::Rcpt
                                }
                                _ => {
                                    writer.write_all(
                                        "503 bad sequence of commands (did you say MAIL FROM?)\r\n"
                                            .as_bytes(),
                                    )?;
                                    writer.flush()?;
                                    SmtpState::Hello
                                }
                            };
                        }

                        "DATA" => {
                            state = match state {
                                SmtpState::Rcpt => {
                                    writer.write_all(
                                        "250 give me the message (. by itself to end)\r\n"
                                            .as_bytes(),
                                    )?;
                                    writer.flush()?;
                                    SmtpState::Data
                                }
                                _ => {
                                    writer.write_all(
                                        "503 bad sequence of commands (did you say RCPT TO?)\r\n"
                                            .as_bytes(),
                                    )?;
                                    writer.flush()?;
                                    state
                                }
                            };
                        }

                        _ => {
                            writer.write_all("502 command not implemented\r\n".as_bytes())?;
                            writer.flush()?;
                        }
                    }
                }
            },
        }
    }
    Ok(())
}
