// SPDX-License-Identifier: BSD-2-Clause-Patent
//
// io_hook.bpf.c — eBPF I/O latency hook for m-store
//
// This program attaches to the NVMe completion path and records per-queue
// latency histograms exportable via BPF maps.
//
// TODO: implement using libbpf skeleton and attach via bpf_program__attach_tracepoint.

#include <linux/bpf.h>
#include <bpf/bpf_helpers.h>

struct {
    __uint(type, BPF_MAP_TYPE_HISTOGRAM);
    __uint(max_entries, 64);
    __type(key, __u32);
    __type(value, __u64);
} latency_hist SEC(".maps");

SEC("tracepoint/nvme/nvme_sq")
int trace_nvme_sq(void *ctx)
{
    /* TODO: record submission timestamp keyed by queue ID. */
    return 0;
}

SEC("tracepoint/nvme/nvme_complete_rq")
int trace_nvme_complete(void *ctx)
{
    /* TODO: compute delta, update latency_hist. */
    return 0;
}

char LICENSE[] SEC("license") = "Dual BSD/GPL";
