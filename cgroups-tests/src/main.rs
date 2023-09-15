use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::thread;
use std::time::Instant;
use gettid;

fn main() {

    cleanup_cgroup();
    setup_cgroup();

    // set the weight of the main process to 80%
    fs::write("/sys/fs/cgroup/my_cgroup/cpu.weight", "8000")
        .expect("Failed to set CPU weight for main process");

    const THREAD_WEIGHTS: [(u32, &str); 3] = [(10, "A"), (30, "B"), (40, "C")];

    for &(weight, thread_id) in &THREAD_WEIGHTS {
        // Create subdirectories for each thread and set up their respective cgroup configurations
        let cgroup_dir = format!("/sys/fs/cgroup/my_cgroup/thread_{}", thread_id);
        fs::create_dir(&cgroup_dir).expect("Failed to create thread cgroup");
        fs::write(format!("{}/cgroup.type", cgroup_dir), "threaded").expect("Failed to set cgroup type");
        fs::write(format!("{}/cpu.weight", cgroup_dir), &format!("{}", weight * 100))
            .expect("Failed to set CPU weight");
    }
    
    // let weights = [100, 300, 400];
    // let handles: Vec<_> = weights.iter().enumerate().map(|(i, &weight)| {
    //     thread::spawn(move || {
    //         set_thread_weight(i, weight);
    //         let start = Instant::now();
    //         do_work();
    //         println!("Thread {} finished work in {:?}", i, start.elapsed());
    //     })
    // }).collect();

    let handles: Vec<_> = THREAD_WEIGHTS.iter().map(|&(weight, thread_id)| {
        thread::spawn(move || {
            
            // Add this thread to the cgroup
            let cgroup_dir = format!("/sys/fs/cgroup/my_cgroup/thread_{}", thread_id);
            fs::OpenOptions::new()
                .write(true)
                .open(format!("{}/cgroup.threads", cgroup_dir))
                .and_then(|mut file| file.write_all(thread_id.as_bytes()))
                .expect("Failed to add thread to cgroup");

            fs::write(format!("/sys/fs/cgroup/my_cgroup/thread_{}/cpu.weight", thread_id), weight.to_string())
            .expect("Failed to set CPU weight for thread");

            let start = Instant::now();
            do_work();
            println!("Thread {} finished work in {:?}", thread_id, start.elapsed());
        })
    }).collect();

    for handle in handles {
        handle.join().expect("Failed to join thread");
    }
}

fn do_work() {
    let mut x = 0;
    for _ in 0..1_000_000_000 {
        x += 1;
    }
}

fn test1() {
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

    // set a CPU max limit for the cgroup (for example, 10000 us every 50000 us)
    fs::write("/sys/fs/cgroup/my_cgroup/cpu.max", "10000 50000")
        .expect("Failed to set CPU max limit");

    // enable threaded mode to allow adding individual threads to the cgroup
    fs::write("/sys/fs/cgroup/my_cgroup/cgroup.type", "threaded")
        .expect("Failed to set cgroup type to threaded");

    // spawn threads and add them to the cgroup
    let handles: Vec<_> = (0..4).map(|i| {
        thread::spawn(move || {
            // get the thread id as a cgroup v2-compatible string
            let tid = format!("{}", gettid::gettid());

            // add this thread to the cgroup
            fs::OpenOptions::new()
                .write(true)
                .open("/sys/fs/cgroup/my_cgroup/cgroup.threads")
                .and_then(|mut file| file.write_all(tid.as_bytes()))
                .expect("Failed to add thread to cgroup");

            // now this thread is in the cgroup
            let start = Instant::now();

            // perform some CPU intensive work
            do_work();

            println!("Thread {} in cgroup finished work in {:?}", i, start.elapsed());
        })
    }).collect();

    // spawn a thread outside the cgroup to compare
    let outside_handle = thread::spawn(|| {
        let start = Instant::now();

        // perform the same cpu intensive work
        do_work();

        println!("Thread outside cgroup finished work in {:?}", start.elapsed());
    });

    // wait for all threads to finish
    for handle in handles {
        handle.join().expect("Failed to join thread");
    }

    outside_handle.join().expect("Failed to join thread outside cgroup");
}

fn setup_cgroup() {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open("/sys/fs/cgroup/cgroup.subtree_control")
        .expect("Failed to open cgroup.subtree_control");

    file.write_all(b"+cpu\n").expect("Failed to delegate CPU controller");
    file.write_all(b"+cpuset\n").expect("Failed to delegate cpuset controller");

    fs::create_dir("/sys/fs/cgroup/my_cgroup").expect("Failed to create cgroup");
}

fn set_thread_weight(thread_id: usize, weight: u32) {
    let tid = format!("{}", gettid::gettid());

    fs::OpenOptions::new()
        .write(true)
        .open(format!("/sys/fs/cgroup/my_cgroup/thread_{}", thread_id))
        .and_then(|mut file| file.write_all(tid.as_bytes()))
        .expect("Failed to add thread to cgroup");

    fs::write(format!("/sys/fs/cgroup/my_cgroup/thread_{}/cpu.weight", thread_id), weight.to_string())
        .expect("Failed to set CPU weight for thread");
}

fn cleanup_cgroup() {
    let _ = fs::remove_dir("/sys/fs/cgroup/my_cgroup");
}