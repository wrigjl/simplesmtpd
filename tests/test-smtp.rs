use simplesmtpd::SmtpState;

#[test]
fn helo_domain() {
    let mut output = std::io::sink();
    let state =
        simplesmtpd::handle_cmd_helo("HELO domain.com", SmtpState::Start, &mut output).unwrap();
    assert!(matches!(state, SmtpState::Hello));
}

#[test]
fn helo_missing() {
    let mut output = std::io::sink();
    let state = simplesmtpd::handle_cmd_helo("HELO", SmtpState::Start, &mut output).unwrap();
    assert!(matches!(state, SmtpState::Start));
}

#[test]
fn ehlo_domain() {
    let mut output = std::io::sink();
    let state =
        simplesmtpd::handle_cmd_ehlo("EHLO domain.com", SmtpState::Start, &mut output).unwrap();
    assert!(matches!(state, SmtpState::Hello));
}

#[test]
fn ehlo_ip4() {
    let mut output = std::io::sink();
    let state =
        simplesmtpd::handle_cmd_ehlo("EHLO [8.8.8.8]", SmtpState::Start, &mut output).unwrap();
    assert!(matches!(state, SmtpState::Hello));
}

#[test]
fn ehlo_ip6() {
    let mut output = std::io::sink();
    let state =
        simplesmtpd::handle_cmd_ehlo("EHLO [IPv6:ff01::1]", SmtpState::Start, &mut output).unwrap();
    assert!(matches!(state, SmtpState::Hello));
}
