use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::thread;
use std::time::Instant;
use gettid;

fn main() {
    // Open cgroup.subtree_control file in append mode to delegate the CPU and cpuset controllers to the new cgroup (at the root level)
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open("/sys/fs/cgroup/cgroup.subtree_control")
        .expect("Failed to open cgroup.subtree_control");

    // Delegate the CPU and cpuset controllers at root level
    file.write_all(b"+cpu\n").expect("Failed to delegate CPU controller");
    file.write_all(b"+cpuset\n").expect("Failed to delegate cpuset controller");

    // Create a new cgroup in the cgroups v2 hierarchy
    fs::create_dir("/sys/fs/cgroup/my_cgroup").expect("Failed to create cgroup");

    // // Enable the CPU controller for the new cgroup
    // fs::write("/sys/fs/cgroup/my_cgroup/cgroup.subtree_control", "+cpu\n")
    //     .expect("Failed to enable CPU controller");

    // Set a CPU max limit for the cgroup (for example, 10000 us every 50000 us)
    fs::write("/sys/fs/cgroup/my_cgroup/cpu.max", "10000 50000")
        .expect("Failed to set CPU max limit");

    // Enable threaded mode to allow adding individual threads to the cgroup
    fs::write("/sys/fs/cgroup/my_cgroup/cgroup.type", "threaded")
        .expect("Failed to set cgroup type to threaded");

    // Spawn some threads and add them to the cgroup
    let handles: Vec<_> = (0..4).map(|i| {
        thread::spawn(move || {
            // Get the thread id as a cgroup v2-compatible string
            let tid = format!("{}", gettid::gettid());

            // Add this thread to the cgroup
            fs::OpenOptions::new()
                .write(true)
                .open("/sys/fs/cgroup/my_cgroup/cgroup.threads")
                .and_then(|mut file| file.write_all(tid.as_bytes()))
                .expect("Failed to add thread to cgroup");

            // Now this thread is in the cgroup and its CPU usage is limited
            let start = Instant::now();

            // Perform some CPU intensive work
            do_work();

            println!("Thread {} in cgroup finished work in {:?}", i, start.elapsed());
        })
    }).collect();

    // Spawn a thread outside the cgroup to compare
    let outside_handle = thread::spawn(|| {
        let start = Instant::now();

        // Perform the same CPU intensive work
        do_work();

        println!("Thread outside cgroup finished work in {:?}", start.elapsed());
    });

    // Wait for all threads to finish
    for handle in handles {
        handle.join().expect("Failed to join thread");
    }

    outside_handle.join().expect("Failed to join thread outside cgroup");
}

fn do_work() {
    let mut x = 0;
    for _ in 0..1_000_000_000 {
        x += 1;
    }
}