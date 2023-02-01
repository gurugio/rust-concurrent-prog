use nix::sys::epoll::{
    epoll_create1, epoll_ctl, epoll_wait, EpollCreateFlags, EpollEvent, EpollFlags, EpollOp,
};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::TcpListener;
use std::os::unix::io::{AsRawFd, RawFd};

fn main() {
    let epoll_in = EpollFlags::EPOLLIN;
    let epoll_add = EpollOp::EpollCtlAdd;
    let epoll_del = EpollOp::EpollCtlDel;

    let listener = TcpListener::bind("127.0.0.1:10000").unwrap();

    let epfd = epoll_create1(EpollCreateFlags::empty()).unwrap();

    let listen_fd = listener.as_raw_fd();
    let mut ev = EpollEvent::new(epoll_in, listen_fd as u64);
    epoll_ctl(epfd, epoll_add, listen_fd, &mut ev).unwrap();

    let mut fd2buf = HashMap::new();
    let mut events = vec![EpollEvent::empty(); 1024];

    while let Ok(nfds) = epoll_wait(epfd, &mut events, -1) {
        for n in 0..nfds {
            if events[n].data() == listen_fd as u64 {
                if let Ok((stream, _)) = listener.accept() {
                    let fd = stream.as_raw_fd();
                    let stream0 = stream.try_clone().unwrap();
                    let reader = BufReader::new(stream0);
                    let writer = BufWriter::new(stream);

                    fd2buf.insert(fd, (reader, writer));

                    println!("accept: fd = {}", fd);

                    let mut ev = EpollEvent::new(epoll_in, fd as u64);
                    epoll_ctl(epfd, epoll_add, fd, &mut ev).unwrap();
                }
            } else {
                // data receiving
                let fd = events[n].data() as RawFd;
                let (reader, writer) = fd2buf.get_mut(&fd).unwrap();

                let mut buf = String::new();
                let n = reader.read_line(&mut buf).unwrap();

                if n == 0 {
                    let mut ev = EpollEvent::new(epoll_in, fd as u64);
                    epoll_ctl(epfd, epoll_del, fd, &mut ev).unwrap();
                    fd2buf.remove(&fd);
                    println!("closed: fd = {}", fd);
                    continue;
                }

                println!("read: fd = {} buf={}", fd, buf);

                writer.write(buf.as_bytes()).unwrap();
                writer.flush().unwrap();
            }
        }
    }
}
