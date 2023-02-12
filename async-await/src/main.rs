async fn do_sleep(i: i32) {
    let secs = std::time::Duration::from_secs(5);
    println!("{} do_sleep: sleep", i);
    //std::thread::sleep(secs);
    tokio::time::sleep(secs).await;
    println!("{} do_sleep: end", i);
}

#[tokio::main]
async fn main() {
    let mut v = Vec::new();

    for i in 0..64 {
        let t = tokio::spawn(do_sleep(i));
        v.push(t);
    }

    println!("join");
    for h in v {
        _ = h.await;
    }
}
