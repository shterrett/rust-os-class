use std::io;
use std::cmp::Ordering;
use std::net::{ TcpStream, SocketAddr };
use std::collections::BinaryHeap;

#[derive(Eq, PartialEq, Debug)]
enum Priority {
    High,
    Low
}

pub struct WeightedStream {
    weight: usize,
    pub stream: TcpStream,
}

pub type FastLane = BinaryHeap<WeightedStream>;
pub type SlowLane = BinaryHeap<WeightedStream>;

impl PartialEq for WeightedStream {
    fn eq(&self, other: &WeightedStream) ->  bool {
        self.weight.eq(&other.weight)
    }
}

impl Eq for WeightedStream {}

impl Ord for WeightedStream {
    fn cmp(&self, other: &WeightedStream) -> Ordering {
        self.weight.cmp(&other.weight)
    }
}

impl PartialOrd for WeightedStream {
    fn partial_cmp(&self, other: &WeightedStream) -> Option<Ordering> {
        self.weight.partial_cmp(&other.weight)
    }
}

pub fn queues() -> (FastLane, SlowLane) {
    (BinaryHeap::new(), BinaryHeap::new())
}

pub fn schedule(stream: TcpStream, high_queue: &mut FastLane, low_queue: &mut SlowLane) {
    match stream_priority(&stream) {
        Priority::High => high_queue.push(scheduled_stream(stream)),
        Priority::Low => low_queue.push(scheduled_stream(stream))
    }
}

trait IpAddressable {
    fn ip_address(&self) -> io::Result<SocketAddr>;
}

impl IpAddressable for TcpStream {
    fn ip_address(&self) -> io::Result<SocketAddr> {
        self.peer_addr()
    }
}

fn scheduled_stream(stream: TcpStream) -> WeightedStream {
    WeightedStream {
        weight: 1,
        stream: stream,
    }
}

fn stream_priority(stream: &IpAddressable) -> Priority {
    match stream.ip_address() {
        Err(_) => Priority::Low,
        Ok(SocketAddr::V6(_)) => Priority::Low,
        Ok(SocketAddr::V4(address)) => {
            let octets = address.ip().octets();
            let prefix = (octets[0], octets[1]);
            if prefix == (128, 143) || prefix == (137, 54) {
                Priority::High
            } else {
                Priority::Low
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::io;
    use std::net::{
        SocketAddr,
        SocketAddrV4,
        Ipv4Addr,
        SocketAddrV6,
        Ipv6Addr
    };
    use super::{
        IpAddressable,
        Priority,
        stream_priority
    };

    struct FakeStream {
        ip: SocketAddr
    }

    impl IpAddressable for FakeStream {
        fn ip_address(&self) -> io::Result<SocketAddr> {
            Ok(self.ip)
        }
    }

    #[test]
    fn uva_ip_prioritized() {
        let uva_stream = FakeStream {
            ip: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(128, 143, 23, 108), 80))
        };
        let other_stream = FakeStream {
            ip: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(10, 2, 10, 5), 80))
        };
        let v6_stream = FakeStream {
            ip: SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1), 80, 0, 0))
        };

        assert_eq!(stream_priority(&uva_stream), Priority::High);
        assert_eq!(stream_priority(&other_stream), Priority::Low);
        assert_eq!(stream_priority(&v6_stream), Priority::Low);
    }
}
