use std::net::Ipv4Addr;

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
struct Quad {
    src: (Ipv4Addr, u16),
    dst: (Ipv4Addr, u16),
}

enum State {
    CLOSING,
    CLOSED,
    LISTEN,
    ESTABLISHD,
    TIMEWAIT,
}


enum 


