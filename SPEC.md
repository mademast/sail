# Notes on the SMTP specification

### Buffers

A number of buffers need to be kept between commands. These hold the
forward and reverse paths, and the mail data. This can be seen in [4.1.1][smtp411].

<details>
	<summary>RFC Text</summary>

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

</details>

### Line endings

Lines must end CR LF per [2.3.8][smtp238].

<details>
	<summary>RFC Text</summary>

> Lines consist of zero or more data characters terminated by the
> sequence ASCII character "CR" (hex value 0D) followed immediately by
> ASCII character "LF" (hex value 0A).  This termination sequence is
> denoted as <CRLF> in this document.  Conforming implementations MUST
> NOT recognize or generate any other character or character sequence
> as a line terminator.  Limits MAY be imposed on line lengths by
> servers (see Section 4).

</details>

### List of Commands
Commands in RFC 5321. The descriptions are not complete, but should currently
be at least enough to implement these commands correctly enough for an MVP.

#### [HELO][smtp4111]
Used to identity the client. Must be sent before a mail transaction is started.
Prefer `EHLO` for modern client implementation.

This command may appear anywhere.

##### Arguments
The domain of the client.

##### Grammar
```ABNF
helo = "HELO" SP Domain CRLF
```

#### [EHLO][smtp4111]
Used to identify the client. This can be sent instead of `HELO`. The server should
respond with supported extensions.

This command may appear anywhere.

##### Arguments
The domain or address (IPv4/IPv6) of the client.

##### Grammar
```ABNF
ehlo = "EHLO" SP ( Domain / address-literal ) CRLF
```

##### Expected return on success
`250 Domain` if there are no service extensions. If there are
service extensions, `250-Domain` and then `250-KEYWORD` for each extension,
indicating the last one by removing the hyphen: `250 KEYWORD`. 

<details>
	<summary>Full Response Grammar</summary>

```ABNF
ehlo-ok-rsp = ( "250" SP Domain [ SP ehlo-greet ] CRLF )
               / ( "250-" Domain [ SP ehlo-greet ] CRLF
               *( "250-" ehlo-line CRLF )
               "250" SP ehlo-line CRLF )
ehlo-greet = 1*(%d0-9 / %d11-12 / %d14-127)
			 ; string of any characters other than CR or LF

ehlo-line = ehlo-keyword *( SP ehlo-param )

ehlo-keywor = (ALPHA / DIGIT) *(ALPHA / DIGIT / "-")
			  ; additional syntax of ehlo-params depends on
			  ; ehlo-keyword

ehlo-param = 1*(%d33-126)
			 ; any CHAR excluding <SP> and all
			 ; control characters (US-ASCII 0-31 and 127
			 ; inclusive)
```
</details>

##### Change to buffers
On success, clear all buffers as if `RSET` was received.
On failure, leave the buffers as they are.

#### [MAIL][smtp4112]
Used to start a mail transaction.

You must not send another `MAIL` command if
a transaction is open. If a server receives this command while a transaction
is ongoing, the server should return a `503` for commands out of order.

##### Arguments
The reverse path (sender's address).

##### Grammar
```ABNF
mail = "MAIL FROM:" Reverse-path [SP Mail-parameters] CRLF
```

##### Change to buffers
On success, clear all buffers and enter the reverse path into the reverse path buffer.
On failure no buffers should be cleared and the server should stay in the same state.

#### [RCPT][smtp4113]
Used to identify an individual recipient of mail data.

This command may be sent multiple times to indicate multiple recipients.

##### Arguments
The forward path (receiver's address).

##### Grammar
```ABNF
rcpt = "RCPT TO:" ( "<Postmaster@" Domain ">" / "<Postmaster>" /
			Forward-path ) [SP Rcpt-parameters] CRLF
```
Note that, in a departure from the usual rules for
local-parts, the "Postmaster" string shown above is
treated as case-insensitive.

#### [DATA][smtp4114]
Used to indicate that the incoming data is the mail data.

To indicate an end to mail data, a line with a single period is sent. That is,
mail data is known to be finished when the sequence `<CRLF>.<CRLF>` is seen.

##### Grammar
```ABNF
data = "DATA" CRLF
```

##### Expected return on success
Servers should send a `354` which mean "start mail input".

#### [RSET][smtp4115]
Used to reset the SMTP servers state and buffers as if it just received
a `HELO`/`EHLO` command.

This command may be sent at any time.

##### Grammar
```ABNF
rset = "RSET" CRLF
```

##### Expected return on success
Servers must respond with a success (`250`)

##### Change to buffers
All buffers are cleared and the start is reset as if the client has just
introduced itself.

#### [VRFY][smtp4116]
Used to ask the server if the argument identifies a user or mailbox.

##### Arguments
A string for the server to lookup, possibly identifying a user or mailbox.

##### Grammar
```ABNF
vrfy = "VRFY" SP String CRLF
```

#### [EXPN][smtp4117]
Used to ask the server if the argument is a valid mailing list and, if so, to
return the members of that list.

##### Arguments
A string for the server to lookup, possibly identifying a mailing list.

##### Grammar
```ABNF
expn = "EXPN" SP String CRLF
```

#### [HELP][smtp4118]
Used to get helpful information.

##### Arguments
If present, it should be a command name you'd like help text for.

##### Grammar
```ABNF
help = "HELP" [ SP String ] CRLF
```

#### [NOOP][smtp4119]
A no-operation command.

##### Arguments
If present, they should be ignored.

##### Grammar
```ABNF
noop = "NOOP" [ SP String ] CRLF
```

##### Expected response on success
The OK status code, `250`.

#### [QUIT][smtp41110]
The end of the connection.

This command may be sent at any time. Any in-progress transaction should be aborted.

##### Grammar
```ABNF
quit = "QUIT" CRLF
```

##### Expected response on success
The "service closing" status code, `221`.

[smtp238]: https://datatracker.ietf.org/doc/html/rfc5321#section-2.3.8
[smtp411]: https://datatracker.ietf.org/doc/html/rfc5321#section-4.1.1
[smtp4111]: https://datatracker.ietf.org/doc/html/rfc5321#section-4.1.1.1
[smtp4112]: https://datatracker.ietf.org/doc/html/rfc5321#section-4.1.1.2
[smtp4113]: https://datatracker.ietf.org/doc/html/rfc5321#section-4.1.1.3
[smtp4114]: https://datatracker.ietf.org/doc/html/rfc5321#section-4.1.1.4
[smtp4115]: https://datatracker.ietf.org/doc/html/rfc5321#section-4.1.1.5
[smtp4116]: https://datatracker.ietf.org/doc/html/rfc5321#section-4.1.1.6
[smtp4117]: https://datatracker.ietf.org/doc/html/rfc5321#section-4.1.1.7
[smtp4118]: https://datatracker.ietf.org/doc/html/rfc5321#section-4.1.1.8
[smtp4119]: https://datatracker.ietf.org/doc/html/rfc5321#section-4.1.1.9
[smtp41110]: https://datatracker.ietf.org/doc/html/rfc5321#section-4.1.1.10
