# Notes on the Internet Message Format (RFC 5322)

# 2: An Overview
- Lines end CRLF
- The message format is the header block and then, optionally, the body
- A line MUST not be more than 1000 characters total long and SHOULD NOT be more than  80 (lest it "disastrously wrap")

## Header Fields
- In the format `<field name>:<field body>`.
- The field name must be ASCII 33-126, excluding the colon
- A field body may be 32-126 and horizontal tab (ASCII 9).
- A field body MUST NOT include CRLF unless used in folding and unfolding

### Unstructured Header Field Bodies
- A field body that is simply a string with no further processing done
- Can still be folding/unfolding

### Structured Header Field Bodies
- Specific tokens described in sections 3, 4
- May still be folded/unfolded where appropriate.

### Long Header Fields
- Folding whitespace (so that's what it is) is when you have to break a line, because it's too long, so you CRLF and start the next line with whitespace to indicate it is part of the previous line.
- Unfolding is removing any CRLF immediately followed by whitespace
- Unfolded lines are not subject to line length restrictions

## The Message Body
- Simply lines of US-ASCII but...
- CR and LF MUST appear together as CRLF. They MUST NOT appear independently
- Same line length restrictions as mentioned in 2

# 3: Legal Syntax
- Tokens starting with `obs-` refer to obsolete syntax described in 4. You MUST be able to parse these but MUST NOT generate them.

## ABNF Things
From 5234
```abnf
SP    = %x20      ; A space character
HTAB  = %x09      ; A tab charactre
WSP   = SP / HTAB ; White space
VCHAR = %x21-7E   ; Visibile, printable, ASCII
```

- Escaping characters
```abnf
quoted-pair = ("\" (VCHAR / WSP)) / obs-qp
```

