use std::io;
use std::cmp::Ordering;
use std::net::{ SocketAddr };
use std::collections::BinaryHeap;
use std::fs::File;

use request::Request;
use path::Path;

#[derive(Eq, PartialEq, Debug)]
enum Priority {
    High,
    Low
}

pub struct WeightedRequest<R: IpAddressable + Pathable> {
    weight: u64,
    pub request: R,
}

pub type FastLane<R> = BinaryHeap<WeightedRequest<R>>;
pub type SlowLane<R> = BinaryHeap<WeightedRequest<R>>;

impl<R> PartialEq for WeightedRequest<R> where R: IpAddressable + Pathable {
    fn eq(&self, other: &WeightedRequest<R>) ->  bool {
        self.weight.eq(&other.weight)
    }
}

impl<R: IpAddressable +  Pathable> Eq for WeightedRequest<R> {}

impl<R> Ord for WeightedRequest<R> where R: IpAddressable + Pathable {
    fn cmp(&self, other: &WeightedRequest<R>) -> Ordering {
        self.weight.cmp(&other.weight)
    }
}

pub trait IpAddressable {
    fn ip_address(&self) -> io::Result<SocketAddr>;
}

impl IpAddressable for Request {
    fn ip_address(&self) -> io::Result<SocketAddr> {
        self.stream.peer_addr()
    }
}

pub trait Pathable {
    fn path(&self) -> &io::Result<Path>;
}

impl Pathable for Request {
    fn path(&self) -> &io::Result<Path> {
        &self.path
    }
}

impl<R> PartialOrd for WeightedRequest<R> where R: IpAddressable + Pathable {
    fn partial_cmp(&self, other: &WeightedRequest<R>) -> Option<Ordering> {
        self.weight.partial_cmp(&other.weight)
    }
}

pub fn queues<R>() -> (FastLane<R>, SlowLane<R>) where R: IpAddressable + Pathable {
    (BinaryHeap::new(), BinaryHeap::new())
}

pub fn schedule<R>(request: R, high_queue: &mut FastLane<R>, low_queue: &mut SlowLane<R>)
    where R: IpAddressable + Pathable {
    match priority(&request) {
        Priority::High => high_queue.push(scheduled_request(request)),
        Priority::Low => low_queue.push(scheduled_request(request))
    }
}

fn scheduled_request<R>(request: R) -> WeightedRequest<R>
    where R: IpAddressable + Pathable{

    let weight = weight(&request.path());
    WeightedRequest {
        weight: weight,
        request: request,
    }
}

fn weight(req_path: &io::Result<Path>) -> u64 {
    match req_path {
        &Err(_) => 0,
        &Ok(Path::Root) => 1,
        &Ok(Path::RelPath(ref path)) => {
            File::open(path)
                .and_then(|f| f.metadata())
                .map(|data| data.len())
                .map(|size| {
                    if path.ends_with("shtml") {
                        size * 2
                    } else {
                        size
                    }

                })
                .unwrap_or(u64::max_value())
        }
    }
}

fn priority(request: &IpAddressable) -> Priority {
    match request.ip_address() {
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
    use path::Path;
    use super::{
        IpAddressable,
        Pathable,
        Priority,
        priority,
        queues,
        schedule
    };

    struct FakeRequest<'a> {
        name: &'a str,
        ip: SocketAddr,
        path: io::Result<Path>
    }

    impl<'a> IpAddressable for FakeRequest<'a> {
        fn ip_address(&self) -> io::Result<SocketAddr> {
            Ok(self.ip)
        }
    }

    impl<'a> Pathable for FakeRequest<'a> {
        fn path(&self) -> &io::Result<Path> {
            &self.path
        }
    }

    #[test]
    fn uva_ip_prioritized() {
        let uva_stream = FakeRequest {
            name: "UVa",
            ip: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(128, 143, 23, 108), 80)),
            path: Ok(Path::Root)
        };
        let other_stream = FakeRequest {
            name: "Other",
            ip: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(10, 2, 10, 5), 80)),
            path: Ok(Path::Root)
        };
        let v6_stream = FakeRequest {
            name: "V6",
            ip: SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1), 80, 0, 0)),
            path: Ok(Path::Root)
        };

        assert_eq!(priority(&uva_stream), Priority::High);
        assert_eq!(priority(&other_stream), Priority::Low);
        assert_eq!(priority(&v6_stream), Priority::Low);
    }

    #[test]
    fn smaller_files_are_prioritized_within_a_queue() {
        let ip = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(128, 143, 23, 108), 80));
        let error_req = FakeRequest {
            name: "Error",
            ip: ip,
            path: Err(io::Error::new(io::ErrorKind::Other, "Whoops!"))
        };
        let root_req = FakeRequest {
            name: "Root",
            ip: ip,
            path: Ok(Path::Root)
        };
        let small_req = FakeRequest {
            name: "Small",
            ip: ip,
            path: Ok(Path::RelPath("test/small.html".to_string()))
        };
        let big_req = FakeRequest {
            name: "Big",
            ip: ip,
            path: Ok(Path::RelPath("test/large.html".to_string()))
        };

        let (mut fast, mut slow) = queues();

        schedule(error_req, &mut fast, &mut slow);
        schedule(big_req, &mut fast, &mut slow);
        schedule(small_req, &mut fast, &mut slow);
        schedule(root_req, &mut fast, &mut slow);

        let order = fast.into_sorted_vec().iter().map(|wr| wr.request.name).collect::<Vec<&str>>();
        assert_eq!(order, vec!["Error", "Root", "Small", "Big"]);
    }

    #[test]
    fn shtml_files_are_deprioritized_within_a_queue() {
        let ip = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(128, 143, 23, 108), 80));
        let small_req = FakeRequest {
            name: "Small",
            ip: ip,
            path: Ok(Path::RelPath("test/small.html".to_string()))
        };
        let small_shtml_req = FakeRequest {
            name: "Dynamic",
            ip: ip,
            path: Ok(Path::RelPath("test/small.shtml".to_string()))
        };
        let med_req = FakeRequest {
            name: "Medium",
            ip: ip,
            path: Ok(Path::RelPath("test/medium.html".to_string()))
        };

        let (mut fast, mut slow) = queues();

        schedule(small_shtml_req, &mut fast, &mut slow);
        schedule(small_req, &mut fast, &mut slow);
        schedule(med_req, &mut fast, &mut slow);

        let order = fast.into_sorted_vec().iter().map(|wr| wr.request.name).collect::<Vec<&str>>();
        assert_eq!(order, vec!["Small", "Medium", "Dynamic"]);
    }
}
