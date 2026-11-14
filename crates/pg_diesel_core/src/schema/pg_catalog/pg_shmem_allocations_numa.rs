//! Submodule for the `pg_catalog.pg_shmem_allocations_numa` view schema.

diesel::table! {
    /// `pg_catalog.pg_shmem_allocations_numa` — view showing shared memory allocations distributed across NUMA nodes.
    /// Available in `PostgreSQL` 17+.
    pg_catalog.pg_shmem_allocations_numa (name, numa_node) {
        /// The name of the shared memory allocation.
        name -> Text,
        /// ID of NUMA node.
        numa_node -> Integer,
        /// Size of the allocation on this particular NUMA memory node.
        size -> BigInt,
    }
}
