error_chain! {
    foreign_links {
        Io(::std::io::Error);
        Tls(::native_tls::Error);
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

        HostParseFailure(host: String) {
            description("Unable to parse host.")
            display("Unable to parse host: {}", host)
        }
    }
}