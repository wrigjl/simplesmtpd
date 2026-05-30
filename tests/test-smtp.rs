use simplesmtpd::SmtpState;

// Extract the numeric reply code from a response buffer.
fn code(buf: &[u8]) -> u16 {
    std::str::from_utf8(buf).unwrap()[..3].parse().unwrap()
}

// Run a full session through handle_client and return the output.
fn session(input: &str) -> String {
    let mut output = Vec::new();
    simplesmtpd::handle_client(input.as_bytes(), &mut output, "test.example.com").unwrap();
    String::from_utf8(output).unwrap()
}

// Extract reply codes from a multi-line session transcript.
fn session_codes(output: &str) -> Vec<u16> {
    output
        .lines()
        .filter(|l| l.len() >= 3 && l[..3].chars().all(|c| c.is_ascii_digit()))
        .map(|l| l[..3].parse().unwrap())
        .collect()
}

// ── HELO ─────────────────────────────────────────────────────────────────────

#[test]
fn helo_domain() {
    let mut buf = Vec::new();
    let state = simplesmtpd::handle_cmd_helo(
        "HELO domain.com",
        SmtpState::Start,
        &mut buf,
        "test.example.com",
    )
    .unwrap();
    assert!(matches!(state, SmtpState::Hello));
    assert_eq!(code(&buf), 250);
}

#[test]
fn helo_missing_arg() {
    let mut buf = Vec::new();
    let state =
        simplesmtpd::handle_cmd_helo("HELO", SmtpState::Start, &mut buf, "test.example.com")
            .unwrap();
    assert!(matches!(state, SmtpState::Start));
    assert_eq!(code(&buf), 501);
}

#[test]
fn helo_too_many_args() {
    let mut buf = Vec::new();
    let state = simplesmtpd::handle_cmd_helo(
        "HELO a.com b.com",
        SmtpState::Start,
        &mut buf,
        "test.example.com",
    )
    .unwrap();
    assert!(matches!(state, SmtpState::Start));
    assert_eq!(code(&buf), 501);
}

#[test]
fn helo_invalid_domain() {
    let mut buf = Vec::new();
    let state = simplesmtpd::handle_cmd_helo(
        "HELO not_a_domain!",
        SmtpState::Start,
        &mut buf,
        "test.example.com",
    )
    .unwrap();
    assert!(matches!(state, SmtpState::Start));
    assert_eq!(code(&buf), 501);
}

#[test]
fn helo_mid_session_resets_to_hello() {
    let mut buf = Vec::new();
    let state = simplesmtpd::handle_cmd_helo(
        "HELO domain.com",
        SmtpState::Mail,
        &mut buf,
        "test.example.com",
    )
    .unwrap();
    assert!(matches!(state, SmtpState::Hello));
    assert_eq!(code(&buf), 250);
}

// ── EHLO ─────────────────────────────────────────────────────────────────────

#[test]
fn ehlo_domain() {
    let mut buf = Vec::new();
    let state = simplesmtpd::handle_cmd_ehlo(
        "EHLO domain.com",
        SmtpState::Start,
        &mut buf,
        "test.example.com",
    )
    .unwrap();
    assert!(matches!(state, SmtpState::Hello));
}

#[test]
fn ehlo_ip4() {
    let mut buf = Vec::new();
    let state = simplesmtpd::handle_cmd_ehlo(
        "EHLO [8.8.8.8]",
        SmtpState::Start,
        &mut buf,
        "test.example.com",
    )
    .unwrap();
    assert!(matches!(state, SmtpState::Hello));
}

#[test]
fn ehlo_ip6() {
    let mut buf = Vec::new();
    let state = simplesmtpd::handle_cmd_ehlo(
        "EHLO [IPv6:ff01::1]",
        SmtpState::Start,
        &mut buf,
        "test.example.com",
    )
    .unwrap();
    assert!(matches!(state, SmtpState::Hello));
}

#[test]
fn ehlo_missing_arg() {
    let mut buf = Vec::new();
    let state =
        simplesmtpd::handle_cmd_ehlo("EHLO", SmtpState::Start, &mut buf, "test.example.com")
            .unwrap();
    assert!(matches!(state, SmtpState::Start));
    assert_eq!(code(&buf), 501);
}

#[test]
fn ehlo_too_many_args() {
    let mut buf = Vec::new();
    let state = simplesmtpd::handle_cmd_ehlo(
        "EHLO a.com b.com",
        SmtpState::Start,
        &mut buf,
        "test.example.com",
    )
    .unwrap();
    assert!(matches!(state, SmtpState::Start));
    assert_eq!(code(&buf), 501);
}

#[test]
fn ehlo_invalid_domain() {
    let mut buf = Vec::new();
    let state = simplesmtpd::handle_cmd_ehlo(
        "EHLO not_a_domain!",
        SmtpState::Start,
        &mut buf,
        "test.example.com",
    )
    .unwrap();
    assert!(matches!(state, SmtpState::Start));
    assert_eq!(code(&buf), 501);
}

#[test]
fn ehlo_invalid_ipv6() {
    let mut buf = Vec::new();
    let state = simplesmtpd::handle_cmd_ehlo(
        "EHLO [IPv6:not::valid::addr]",
        SmtpState::Start,
        &mut buf,
        "test.example.com",
    )
    .unwrap();
    assert!(matches!(state, SmtpState::Start));
    assert_eq!(code(&buf), 501);
}

#[test]
fn ehlo_mid_session_resets_to_hello() {
    let mut buf = Vec::new();
    let state = simplesmtpd::handle_cmd_ehlo(
        "EHLO domain.com",
        SmtpState::Mail,
        &mut buf,
        "test.example.com",
    )
    .unwrap();
    assert!(matches!(state, SmtpState::Hello));
}

#[test]
fn ehlo_response_advertises_expn() {
    let mut buf = Vec::new();
    simplesmtpd::handle_cmd_ehlo(
        "EHLO domain.com",
        SmtpState::Start,
        &mut buf,
        "test.example.com",
    )
    .unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("EXPN"), "EHLO response must advertise EXPN");
}

// ── MAIL ─────────────────────────────────────────────────────────────────────

#[test]
fn mail_success() {
    let mut buf = Vec::new();
    let state =
        simplesmtpd::handle_cmd_mail("MAIL FROM:<sender@example.com>", SmtpState::Hello, &mut buf)
            .unwrap();
    assert!(matches!(state, SmtpState::Mail));
    assert_eq!(code(&buf), 250);
}

#[test]
fn mail_null_reverse_path() {
    let mut buf = Vec::new();
    let state = simplesmtpd::handle_cmd_mail("MAIL FROM:<>", SmtpState::Hello, &mut buf).unwrap();
    assert!(matches!(state, SmtpState::Mail));
    assert_eq!(code(&buf), 250);
}

#[test]
fn mail_case_insensitive() {
    let mut buf = Vec::new();
    let state =
        simplesmtpd::handle_cmd_mail("mail from:<sender@example.com>", SmtpState::Hello, &mut buf)
            .unwrap();
    assert!(matches!(state, SmtpState::Mail));
    assert_eq!(code(&buf), 250);
}

#[test]
fn mail_requires_hello_first() {
    let mut buf = Vec::new();
    let state =
        simplesmtpd::handle_cmd_mail("MAIL FROM:<sender@example.com>", SmtpState::Start, &mut buf)
            .unwrap();
    assert!(matches!(state, SmtpState::Start));
    assert_eq!(code(&buf), 503);
}

#[test]
fn mail_rejected_when_transaction_in_progress() {
    let mut buf = Vec::new();
    let state =
        simplesmtpd::handle_cmd_mail("MAIL FROM:<sender@example.com>", SmtpState::Mail, &mut buf)
            .unwrap();
    assert!(matches!(state, SmtpState::Mail));
    assert_eq!(code(&buf), 503);
}

#[test]
fn mail_missing_angle_brackets() {
    let mut buf = Vec::new();
    let state =
        simplesmtpd::handle_cmd_mail("MAIL FROM:sender@example.com", SmtpState::Hello, &mut buf)
            .unwrap();
    assert!(matches!(state, SmtpState::Hello));
    assert_eq!(code(&buf), 501);
}

#[test]
fn mail_empty_arg() {
    let mut buf = Vec::new();
    let state = simplesmtpd::handle_cmd_mail("MAIL FROM:", SmtpState::Hello, &mut buf).unwrap();
    assert!(matches!(state, SmtpState::Hello));
    assert_eq!(code(&buf), 501);
}

#[test]
fn mail_wrong_syntax() {
    let mut buf = Vec::new();
    let state =
        simplesmtpd::handle_cmd_mail("MAIL TO:<sender@example.com>", SmtpState::Hello, &mut buf)
            .unwrap();
    assert!(matches!(state, SmtpState::Hello));
    assert_eq!(code(&buf), 501);
}

// ── RCPT ─────────────────────────────────────────────────────────────────────

#[test]
fn rcpt_success() {
    let mut buf = Vec::new();
    let state =
        simplesmtpd::handle_cmd_rcpt("RCPT TO:<recipient@example.com>", SmtpState::Mail, &mut buf)
            .unwrap();
    assert!(matches!(state, SmtpState::Rcpt));
    assert_eq!(code(&buf), 250);
}

#[test]
fn rcpt_multiple_recipients() {
    let mut buf = Vec::new();
    let state =
        simplesmtpd::handle_cmd_rcpt("RCPT TO:<second@example.com>", SmtpState::Rcpt, &mut buf)
            .unwrap();
    assert!(matches!(state, SmtpState::Rcpt));
    assert_eq!(code(&buf), 250);
}

#[test]
fn rcpt_case_insensitive() {
    let mut buf = Vec::new();
    let state =
        simplesmtpd::handle_cmd_rcpt("rcpt to:<recipient@example.com>", SmtpState::Mail, &mut buf)
            .unwrap();
    assert!(matches!(state, SmtpState::Rcpt));
    assert_eq!(code(&buf), 250);
}

#[test]
fn rcpt_requires_mail_first() {
    let mut buf = Vec::new();
    let state = simplesmtpd::handle_cmd_rcpt(
        "RCPT TO:<recipient@example.com>",
        SmtpState::Hello,
        &mut buf,
    )
    .unwrap();
    assert!(matches!(state, SmtpState::Hello));
    assert_eq!(code(&buf), 503);
}

#[test]
fn rcpt_missing_angle_brackets() {
    let mut buf = Vec::new();
    let state =
        simplesmtpd::handle_cmd_rcpt("RCPT TO:recipient@example.com", SmtpState::Mail, &mut buf)
            .unwrap();
    assert!(matches!(state, SmtpState::Mail));
    assert_eq!(code(&buf), 501);
}

#[test]
fn rcpt_empty_arg() {
    let mut buf = Vec::new();
    let state = simplesmtpd::handle_cmd_rcpt("RCPT TO:", SmtpState::Mail, &mut buf).unwrap();
    assert!(matches!(state, SmtpState::Mail));
    assert_eq!(code(&buf), 501);
}

#[test]
fn rcpt_wrong_syntax() {
    let mut buf = Vec::new();
    let state = simplesmtpd::handle_cmd_rcpt(
        "RCPT FROM:<recipient@example.com>",
        SmtpState::Mail,
        &mut buf,
    )
    .unwrap();
    assert!(matches!(state, SmtpState::Mail));
    assert_eq!(code(&buf), 501);
}

// ── DATA ─────────────────────────────────────────────────────────────────────

#[test]
fn data_success() {
    let mut buf = Vec::new();
    let state = simplesmtpd::handle_cmd_data("DATA", SmtpState::Rcpt, &mut buf).unwrap();
    assert!(matches!(state, SmtpState::Data));
    assert_eq!(code(&buf), 354);
}

#[test]
fn data_requires_rcpt_first() {
    let mut buf = Vec::new();
    let state = simplesmtpd::handle_cmd_data("DATA", SmtpState::Hello, &mut buf).unwrap();
    assert!(matches!(state, SmtpState::Hello));
    assert_eq!(code(&buf), 503);
}

#[test]
fn data_requires_rcpt_not_just_mail() {
    let mut buf = Vec::new();
    let state = simplesmtpd::handle_cmd_data("DATA", SmtpState::Mail, &mut buf).unwrap();
    assert!(matches!(state, SmtpState::Mail));
    assert_eq!(code(&buf), 503);
}

#[test]
fn data_with_arguments_rejected() {
    let mut buf = Vec::new();
    let state = simplesmtpd::handle_cmd_data("DATA somearg", SmtpState::Rcpt, &mut buf).unwrap();
    assert!(matches!(state, SmtpState::Rcpt));
    assert_eq!(code(&buf), 501);
}

// ── RSET ─────────────────────────────────────────────────────────────────────

#[test]
fn rset_from_start_stays_start() {
    let mut buf = Vec::new();
    let state = simplesmtpd::handle_cmd_rset("RSET", SmtpState::Start, &mut buf).unwrap();
    assert!(matches!(state, SmtpState::Start));
    assert_eq!(code(&buf), 250);
}

#[test]
fn rset_from_hello_stays_hello() {
    let mut buf = Vec::new();
    let state = simplesmtpd::handle_cmd_rset("RSET", SmtpState::Hello, &mut buf).unwrap();
    assert!(matches!(state, SmtpState::Hello));
    assert_eq!(code(&buf), 250);
}

#[test]
fn rset_from_mail_returns_to_hello() {
    let mut buf = Vec::new();
    let state = simplesmtpd::handle_cmd_rset("RSET", SmtpState::Mail, &mut buf).unwrap();
    assert!(matches!(state, SmtpState::Hello));
    assert_eq!(code(&buf), 250);
}

#[test]
fn rset_from_rcpt_returns_to_hello() {
    let mut buf = Vec::new();
    let state = simplesmtpd::handle_cmd_rset("RSET", SmtpState::Rcpt, &mut buf).unwrap();
    assert!(matches!(state, SmtpState::Hello));
    assert_eq!(code(&buf), 250);
}

#[test]
fn rset_with_arguments_rejected() {
    let mut buf = Vec::new();
    let state = simplesmtpd::handle_cmd_rset("RSET somearg", SmtpState::Mail, &mut buf).unwrap();
    assert!(matches!(state, SmtpState::Mail));
    assert_eq!(code(&buf), 501);
}

// ── VRFY ─────────────────────────────────────────────────────────────────────

#[test]
fn vrfy_does_not_change_state() {
    let mut buf = Vec::new();
    let state =
        simplesmtpd::handle_cmd_vrfy("VRFY user@example.com", SmtpState::Hello, &mut buf).unwrap();
    assert!(matches!(state, SmtpState::Hello));
    assert_eq!(code(&buf), 252);
}

#[test]
fn vrfy_does_not_corrupt_mail_state() {
    // Regression: VRFY used to transition state to Mail regardless of prior state.
    let mut buf = Vec::new();
    let state =
        simplesmtpd::handle_cmd_vrfy("VRFY user@example.com", SmtpState::Mail, &mut buf).unwrap();
    assert!(matches!(state, SmtpState::Mail));
    assert_eq!(code(&buf), 252);
}

#[test]
fn vrfy_usable_before_ehlo() {
    let mut buf = Vec::new();
    let state =
        simplesmtpd::handle_cmd_vrfy("VRFY user@example.com", SmtpState::Start, &mut buf).unwrap();
    assert!(matches!(state, SmtpState::Start));
    assert_eq!(code(&buf), 252);
}

#[test]
fn vrfy_missing_arg() {
    let mut buf = Vec::new();
    let state = simplesmtpd::handle_cmd_vrfy("VRFY", SmtpState::Hello, &mut buf).unwrap();
    assert!(matches!(state, SmtpState::Hello));
    assert_eq!(code(&buf), 501);
}

// ── EXPN ─────────────────────────────────────────────────────────────────────

#[test]
fn expn_does_not_change_state() {
    let mut buf = Vec::new();
    let state = simplesmtpd::handle_cmd_expn("EXPN somelist", SmtpState::Hello, &mut buf).unwrap();
    assert!(matches!(state, SmtpState::Hello));
    assert_eq!(code(&buf), 252);
}

#[test]
fn expn_does_not_corrupt_mail_state() {
    // Regression: EXPN used to transition state to Mail regardless of prior state.
    let mut buf = Vec::new();
    let state = simplesmtpd::handle_cmd_expn("EXPN somelist", SmtpState::Mail, &mut buf).unwrap();
    assert!(matches!(state, SmtpState::Mail));
    assert_eq!(code(&buf), 252);
}

#[test]
fn expn_usable_before_ehlo() {
    let mut buf = Vec::new();
    let state = simplesmtpd::handle_cmd_expn("EXPN somelist", SmtpState::Start, &mut buf).unwrap();
    assert!(matches!(state, SmtpState::Start));
    assert_eq!(code(&buf), 252);
}

#[test]
fn expn_missing_arg() {
    let mut buf = Vec::new();
    let state = simplesmtpd::handle_cmd_expn("EXPN", SmtpState::Hello, &mut buf).unwrap();
    assert!(matches!(state, SmtpState::Hello));
    assert_eq!(code(&buf), 501);
}

// ── NOOP ─────────────────────────────────────────────────────────────────────

#[test]
fn noop_preserves_state() {
    let mut buf = Vec::new();
    let state = simplesmtpd::handle_cmd_noop(SmtpState::Mail, &mut buf).unwrap();
    assert!(matches!(state, SmtpState::Mail));
    assert_eq!(code(&buf), 250);
}

#[test]
fn noop_usable_before_ehlo() {
    let mut buf = Vec::new();
    let state = simplesmtpd::handle_cmd_noop(SmtpState::Start, &mut buf).unwrap();
    assert!(matches!(state, SmtpState::Start));
    assert_eq!(code(&buf), 250);
}

// ── HELP ─────────────────────────────────────────────────────────────────────

#[test]
fn help_returns_214() {
    let mut buf = Vec::new();
    let state = simplesmtpd::handle_cmd_help(SmtpState::Hello, &mut buf).unwrap();
    assert!(matches!(state, SmtpState::Hello));
    assert_eq!(code(&buf), 214);
}

#[test]
fn help_usable_before_ehlo() {
    let mut buf = Vec::new();
    let state = simplesmtpd::handle_cmd_help(SmtpState::Start, &mut buf).unwrap();
    assert!(matches!(state, SmtpState::Start));
    assert_eq!(code(&buf), 214);
}

// ── QUIT ─────────────────────────────────────────────────────────────────────

#[test]
fn quit_success() {
    let mut buf = Vec::new();
    let state =
        simplesmtpd::handle_cmd_quit("QUIT", SmtpState::Hello, &mut buf, "test.example.com")
            .unwrap();
    assert!(matches!(state, SmtpState::Quit));
    assert_eq!(code(&buf), 221);
}

#[test]
fn quit_221_includes_domain() {
    let mut buf = Vec::new();
    simplesmtpd::handle_cmd_quit("QUIT", SmtpState::Hello, &mut buf, "test.example.com").unwrap();
    let response = String::from_utf8(buf).unwrap();
    // RFC 5321 §4.2.2: 221 response must begin with the server domain.
    assert!(response.starts_with("221 ") && response.len() > 4 && !response[4..].starts_with(' '));
}

#[test]
fn quit_with_arguments_rejected() {
    let mut buf = Vec::new();
    let state =
        simplesmtpd::handle_cmd_quit("QUIT now", SmtpState::Hello, &mut buf, "test.example.com")
            .unwrap();
    assert!(matches!(state, SmtpState::Hello));
    assert_eq!(code(&buf), 501);
}

// ── Integration: handle_client ────────────────────────────────────────────────

#[test]
fn session_banner_is_220() {
    let out = session("QUIT\r\n");
    assert!(out.starts_with("220 "), "greeting must be 220");
}

#[test]
fn session_full_transaction() {
    let input = "EHLO client.example.com\r\n\
                 MAIL FROM:<sender@example.com>\r\n\
                 RCPT TO:<recipient@example.com>\r\n\
                 DATA\r\n\
                 Subject: test\r\n\
                 \r\n\
                 Hello.\r\n\
                 .\r\n\
                 QUIT\r\n";
    let codes = session_codes(&session(input));
    // 220 banner; 250×3 EHLO (domain line + EXPN + HELP); 250 MAIL; 250 RCPT; 354 DATA; 250 accepted; 221 QUIT
    assert_eq!(codes, vec![220, 250, 250, 250, 250, 250, 354, 250, 221]);
}

#[test]
fn session_multiple_transactions() {
    // After DATA end, server returns to Hello state and accepts a new MAIL.
    let input = "EHLO client.example.com\r\n\
                 MAIL FROM:<a@example.com>\r\n\
                 RCPT TO:<b@example.com>\r\n\
                 DATA\r\n\
                 first message\r\n\
                 .\r\n\
                 MAIL FROM:<c@example.com>\r\n\
                 RCPT TO:<d@example.com>\r\n\
                 DATA\r\n\
                 second message\r\n\
                 .\r\n\
                 QUIT\r\n";
    let codes = session_codes(&session(input));
    // 220; 250×3 EHLO; 250 MAIL; 250 RCPT; 354; 250; 250 MAIL; 250 RCPT; 354; 250; 221
    assert_eq!(
        codes,
        vec![220, 250, 250, 250, 250, 250, 354, 250, 250, 250, 354, 250, 221]
    );
}

#[test]
fn session_ehlo_mid_transaction_resets() {
    // EHLO mid-session must reset state, allowing MAIL to follow immediately.
    let input = "EHLO client.example.com\r\n\
                 MAIL FROM:<a@example.com>\r\n\
                 EHLO client.example.com\r\n\
                 MAIL FROM:<b@example.com>\r\n\
                 RCPT TO:<c@example.com>\r\n\
                 DATA\r\n\
                 hi\r\n\
                 .\r\n\
                 QUIT\r\n";
    let codes = session_codes(&session(input));
    // 220; 250×3 EHLO; 250 MAIL; 250×3 EHLO reset; 250 MAIL; 250 RCPT; 354; 250; 221
    assert_eq!(
        codes,
        vec![220, 250, 250, 250, 250, 250, 250, 250, 250, 250, 354, 250, 221]
    );
}

#[test]
fn session_unknown_command_returns_500() {
    let input = "EHLO client.example.com\r\nFOO\r\nQUIT\r\n";
    let codes = session_codes(&session(input));
    // 220; 250×3 EHLO; 500 FOO; 221 QUIT
    assert_eq!(codes, vec![220, 250, 250, 250, 500, 221]);
}

#[test]
fn session_rset_aborts_transaction() {
    let input = "EHLO client.example.com\r\n\
                 MAIL FROM:<a@example.com>\r\n\
                 RCPT TO:<b@example.com>\r\n\
                 RSET\r\n\
                 MAIL FROM:<c@example.com>\r\n\
                 RCPT TO:<d@example.com>\r\n\
                 DATA\r\n\
                 hi\r\n\
                 .\r\n\
                 QUIT\r\n";
    let codes = session_codes(&session(input));
    // 220; 250×3 EHLO; 250 MAIL; 250 RCPT; 250 RSET; 250 MAIL; 250 RCPT; 354; 250; 221
    assert_eq!(
        codes,
        vec![220, 250, 250, 250, 250, 250, 250, 250, 250, 354, 250, 221]
    );
}
