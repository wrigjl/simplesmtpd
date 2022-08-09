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
    net::{TcpListener, TcpStream},
    thread,
};

fn main() -> std::io::Result<()> {
    let (mut tx, rx) = spmc::channel::<TcpStream>();
    let mut handles = Vec::new();

    let listener = TcpListener::bind("0.0.0.0:8025")?;

    for _ in 0..20 {
        let rx = rx.clone();
        handles.push(thread::spawn(move || {
            let msg = rx.recv().unwrap();
            simplesmtpd::handle_client(&msg).unwrap();
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
