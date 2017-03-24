// TODO: Come back and document this.
#![allow(missing_docs)]

error_chain! {
    foreign_links {
        Io(::std::io::Error);
        Tls(::native_tls::Error);
        Utf8(::std::string::FromUtf8Error);
    }

    errors { 
        Unexpected {
            description("An unexpected error occurred.")
            display("An unexpected error occurred.")
        }

        ConnectionReset {
            description("The connection was reset by the remote host.")
            display("The connection was reset by the remote host.")
        }

        UnexpectedEndOfInput {
            description("Encountered unexpected end of input while reading message from server.")
            display("Encountered unexpected end of input while reading message from server.")
        }

        InputTooLong(message: String) {
            description("The input was too long.")
            display("{}", message)
        }
    }
}