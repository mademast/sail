# Notes on the SMTP specification

### Buffers

A number of buffers need to be kept between commands. These hold the
forward and reverse paths, and the mail data. This can be seen in [4.1.1][smtp411].

> A mail transaction involves several data objects that are
> communicated as arguments to different commands.  The reverse-path is
> the argument of the MAIL command, the forward-path is the argument of
> the RCPT command, and the mail data is the argument of the DATA
> command.  These arguments or data objects must be transmitted and
> held, pending the confirmation communicated by the end of mail data
> indication that finalizes the transaction.  The model for this is
> that distinct buffers are provided to hold the types of data objects;
> that is, there is a reverse-path buffer, a forward-path buffer, and a
> mail data buffer.  Specific commands cause information to be appended
> to a specific buffer, or cause one or more buffers to be cleared.

### Line endings

Lines must end CR LF per [2.3.8][smtp238].

> Lines consist of zero or more data characters terminated by the
> sequence ASCII character "CR" (hex value 0D) followed immediately by
> ASCII character "LF" (hex value 0A).  This termination sequence is
> denoted as <CRLF> in this document.  Conforming implementations MUST
> NOT recognize or generate any other character or character sequence
> as a line terminator.  Limits MAY be imposed on line lengths by
> servers (see Section 4).

[smtp238]: https://datatracker.ietf.org/doc/html/rfc5321#section-2.3.8
[smtp411]: https://datatracker.ietf.org/doc/html/rfc5321#section-4.1.1