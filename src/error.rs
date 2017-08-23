// TODO: Come back and document this.
#![allow(missing_docs)]

#[cfg(not(feature = "tls"))]
error_chain! {
    foreign_links {
        Io(::std::io::Error);
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
    }

    links {
        Pircolate(::pircolate::error::Error, ::pircolate::error::ErrorKind);
    }
}

#[cfg(feature = "tls")]
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
    }

    links {
        Pircolate(::pircolate::error::Error, ::pircolate::error::ErrorKind);
    }
}
