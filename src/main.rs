extern crate xenstat;
use xenstat::Xen;

extern crate clap;
use clap::App;
// use clap::{App, Arg};

extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;

use std::process::exit;
use std::f64;

#[derive(Serialize)]
struct Output {
    total_mem: u64,
    used_mem: u64,
    free_mem: u64,
    cpus: u32,
    cpu_hz: u64,
    domains: Vec<DomainOutput>,
}

#[derive(Serialize)]
struct DomainOutput {
    name: String,
    // state: DomainState,
    state: String,
    cur_mem: u64,
    max_mem: u64,
    vcpus: Vec<Cpu>,
    nets: Vec<Network>,
    vbds: Vec<BlockDevice>,
    ssid: u32,
}

#[derive(Serialize)]
struct Cpu {
    sec: u64,
}

#[derive(Serialize)]
struct Network {
    id: u32,
    rx: u64,
    tx: u64,
}

#[derive(Serialize)]
struct BlockDevice {
    dev: u32,
    oo: u64,
    rd: u64,
    wr: u64,
}

fn main() {
    let _matches = App::new("xentop-json")
        .get_matches();

    let xen = match Xen::new() {
        Some(xen) => xen,
        None => {
            eprintln!("Error connecting to Xen");
            exit(1);
        },
    };

    let mut domains: Vec<DomainOutput> = Vec::new();
    for i in 0..xen.num_domains() {
        let domain = xen.domain(i);
        domains.push(
            DomainOutput {
                name: domain.name(),
                state: domain.state().print(),
                cur_mem: domain.cur_mem(),
                max_mem: domain.max_mem(),
                vcpus: (0..domain.num_vcpus()).map(|i| domain.vcpu(i)).map(|c| Cpu { sec: f64::round(c.ns() as f64 / 1000000000f64) as u64 } ).collect(),
                nets: (0..domain.num_networks()).map(|i| domain.network(i)).map(|n| Network { id: n.id(), rx: n.rbytes(), tx: n.tbytes() }).collect(),
                vbds: (0..domain.num_vbds()).map(|i| domain.vbd(i)).map(|v| BlockDevice { dev: v.vbd_dev(), oo: v.oo_reqs(), rd: v.rd_reqs(), wr: v.wr_reqs() }).collect(),
                ssid: domain.ssid(),
            }
        );
    }

    let total = xen.total_memory();
    let free = xen.free_memory();
    let used = total - free;

    let output = Output {
        total_mem: total,
        used_mem: used,
        free_mem: free,
        cpus: xen.num_cpus(),
        cpu_hz: xen.cpu_hz(),
        domains: domains,
    };

    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}
