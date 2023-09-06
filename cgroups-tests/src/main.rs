use std::fs;
use std::io::Write;
use std::thread;

fn main() {
    // Delegate the CPU controller to the new cgroup (at the root level)
    fs::write("/sys/fs/cgroup/cgroup.subtree_control", "cpu")
        .expect("Failed to delegate CPU controller");

    // Create a new cgroup in the cgroups v2 hierarchy
    fs::create_dir("/sys/fs/cgroup/my_cgroup").expect("Failed to create cgroup");

    // Enable the CPU controller for the new cgroup
    fs::write("/sys/fs/cgroup/my_cgroup/cgroup.subtree_control", "cpu")
        .expect("Failed to enable CPU controller");

    // Set a CPU max limit for the cgroup (for example, 10000 us every 50000 us)
    fs::write("/sys/fs/cgroup/my_cgroup/cpu.max", "10000 50000")
        .expect("Failed to set CPU max limit");

    // Enable threaded mode to allow adding individual threads to the cgroup
    fs::write("/sys/fs/cgroup/my_cgroup/cgroup.type", "threaded")
        .expect("Failed to set cgroup type to threaded");

    // Spawn some threads and add them to the cgroup
    let handles: Vec<_> = (0..4).map(|_| {
        thread::spawn(|| {
            // Get the thread id as a cgroup v2-compatible string
            let tid = format!("{}", gettid::gettid());

            // Add this thread to the cgroup
            fs::OpenOptions::new()
                .write(true)
                .open("/sys/fs/cgroup/my_cgroup/cgroup.threads")
                .and_then(|mut file| file.write_all(tid.as_bytes()))
                .expect("Failed to add thread to cgroup");

            // Now this thread is in the cgroup and its CPU usage is limited
            loop {
                // Simulate some CPU work
            }
        })
    }).collect();

    // Wait for all threads to finish (they won't, so this will block forever)
    for handle in handles {
        handle.join().expect("Failed to join thread");
    }
}

