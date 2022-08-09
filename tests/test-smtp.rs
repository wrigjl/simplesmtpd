use simplesmtpd::SmtpState;

#[test]
fn helo_domain() {
    let mut output = std::io::sink();
    let state =
        simplesmtpd::handle_cmd_helo("HELO domain.com", SmtpState::Start, &mut output).unwrap();

    // Ensure we switched from start -> hello

    assert!(if let SmtpState::Hello = state {
        true
    } else {
        false
    });
}

#[test]
fn helo_missing() {
    let mut output = std::io::sink();
    let state = simplesmtpd::handle_cmd_helo("HELO", SmtpState::Start, &mut output).unwrap();

    // Ensure we stayed in start state

    assert!(if let SmtpState::Start = state {
        true
    } else {
        false
    });
}
