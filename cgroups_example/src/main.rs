use cgroups_rs::*;
use cgroups_rs::cgroup_builder::*;

fn main() {
    // Acquire a handle for the cgroup hierarchy.
    let hier = cgroups_rs::hierarchies::auto();

    // Use the builder pattern (see the documentation to create the control group)
    //
    // This creates a control group named "example" in the V1 hierarchy.
    let cg: Cgroup = CgroupBuilder::new("example")
        .cpu()
        .shares(85)
        .done()
        .build(hier)
        .unwrap();

    // Now `cg` is a control group that gets 85% of the CPU time in relative to
    // other control groups.

    // Get a handle to the CPU controller.
    let cpus: &cgroups_rs::cpu::CpuController = cg.controller_of().unwrap();
    cpus.add_task(&CgroupPid::from(1234u64)).unwrap();

    // [...]

    // Finally, clean up and delete the control group.
    cg.delete().unwrap();

    // Note that `Cgroup` does not implement `Drop` and therefore when the
    // structure is dropped, the Cgroup will stay around. This is because, later
    // you can then re-create the `Cgroup` using `load()`. We aren't too set on
    // this behavior, so it might change in the future. Rest assured, it will be a
    // major version change.
}