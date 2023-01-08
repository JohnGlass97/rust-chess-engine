use std::thread;

fn sum_range(start: i64, end: i64) -> i64 {
    let mut sum: i64 = 0;
    for i in start..end {
        sum += i;
    }
    sum
}

fn sum_to_threaded(n: i64) -> i64 {
    let thread_count: usize = 64;
    let q = n / (thread_count as i64);
    let mut sum = 0;
    let mut endpoints = Vec::new();

    let mut val = 0;
    for _ in 0..thread_count {
        endpoints.push(val);
        val += q;
    }
    endpoints.push(n + 1);

    let mut handles = Vec::new();

    for i in 0..thread_count {
        let s = endpoints[i];
        let e = endpoints[i + 1];
        let handle = thread::spawn(move || sum_range(s, e));
        handles.push(handle);
    }

    for h in handles {
        sum += h.join().unwrap()
    }
    sum
}

pub fn threading_test() {
    let mut sum = 0;

    let to = 1_000_000;

    println!("Multi threaded:");
    timeit!({
        sum = sum_to_threaded(to);
    });

    println!("Result: {}", sum);

    println!();

    sum = 0;

    println!("Single threaded:");
    timeit!({
        sum = sum_range(1, to + 1);
    });
    println!("Result: {}", sum);
}
