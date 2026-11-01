//! Synthetic metrics and processes for screenshots (`MACMON_DEMO=1`): the same frames on every
//! run, so captures compare one change at a time. Dev only.

use std::f32::consts::TAU;

use macmon::{
  CpuCoreMetrics, CpuTierInfo, CpuTierMetrics, FanMetric, MemMetrics, Metrics, SocInfo, TempMetrics,
};

use super::kill::ProcSys;
use crate::fds::{Kind, OpenFile};
use crate::procs::ProcInfo;

/// Samples drawn before the first frame, so graphs start full.
pub(super) const HISTORY: u32 = 240;

pub(super) fn enabled() -> bool {
  std::env::var_os("MACMON_DEMO").is_some()
}

pub(super) fn soc() -> SocInfo {
  let tier = |label: &str, cores| CpuTierInfo { label: label.to_string(), cores, freqs: vec![] };
  SocInfo {
    chip_name: "Apple M4 Pro".to_string(),
    mac_model: "Mac16,8".to_string(),
    memory_gb: 48,
    cpu_tiers: vec![tier("E", 4), tier("P", 10)],
    gpu_cores: 20,
    ..Default::default()
  }
}

/// Deterministic noise in `0.0..1.0`.
fn noise(seed: u32) -> f32 {
  let x = seed.wrapping_mul(0x9E37_79B9) ^ 0x85EB_CA6B;
  let x = (x ^ (x >> 15)).wrapping_mul(0x2C1B_3C6D);
  (x ^ (x >> 12)) as f32 / u32::MAX as f32
}

/// A load in `0.0..1.0` at sample `t`: slow waves, a build-like burst every 90 samples, noise.
fn load(t: u32, seed: u32, base: f32) -> f32 {
  let t_f = t as f32;
  let wave = 0.5 + 0.5 * (t_f * TAU / 60.0 + seed as f32).sin();
  let phase = (t + seed * 7) % 90;
  let burst = if (40..70).contains(&phase) { 0.55 } else { 0.0 };
  (base + 0.15 * wave + burst + 0.12 * noise(t * 31 + seed)).clamp(0.02, 0.98)
}

pub(super) fn metrics(t: u32) -> Metrics {
  let e = load(t, 1, 0.1) * 0.75;
  let p = load(t, 2, 0.12);
  let gpu = load(t + 20, 3, 0.05) * 0.8;
  let ane = if (t / 45) % 3 == 1 { 0.3 + 0.2 * noise(t) } else { 0.0 };
  let cores = |n: usize, ratio: f32, seed: u32| -> Vec<CpuCoreMetrics> {
    (0..n)
      .map(|i| {
        let r = (ratio * (0.6 + 0.8 * noise(t * 13 + seed + i as u32 * 101))).clamp(0.0, 1.0);
        CpuCoreMetrics {
          core_id: i,
          freq_mhz: 1000 + (3000.0 * r) as u32,
          scaled_ratio: r,
          active_ratio: r,
          ..Default::default()
        }
      })
      .collect()
  };
  let tier = |label: &str, mhz: f32, ratio: f32, n: usize, seed: u32| CpuTierMetrics {
    label: label.to_string(),
    freq_mhz: (mhz * (0.6 + 0.4 * ratio)) as u32,
    scaled_ratio: ratio,
    active_ratio: (ratio * 1.1).min(1.0),
    cores: cores(n, ratio, seed),
  };
  let (cpu_w, gpu_w, ane_w) = (1.0 + 22.0 * p + 3.0 * e, 0.3 + 14.0 * gpu, 2.8 * ane);
  let all = cpu_w + gpu_w + ane_w;
  let ram_used = 0.62 + 0.08 * (t as f32 / 200.0).sin();
  Metrics {
    temp: TempMetrics {
      cpu_temp_avg: Some(48.0 + 30.0 * p),
      gpu_temp_avg: Some(44.0 + 22.0 * gpu),
    },
    memory: MemMetrics {
      ram_total: 48 << 30,
      ram_usage: ((48u64 << 30) as f32 * ram_used) as u64,
      swap_total: 4 << 30,
      swap_usage: 3 << 28,
    },
    fans: vec![FanMetric {
      name: "fan0".to_string(),
      rpm: 1300 + (2200.0 * p) as u32,
      max_rpm: Some(4900),
    }],
    cpu_tiers: vec![tier("E", 2592.0, e, 4, 10), tier("P", 4512.0, p, 10, 20)],
    gpu_freq_mhz: 400 + (1178.0 * gpu) as u32,
    gpu_scaled_ratio: gpu,
    gpu_active_ratio: (gpu * 1.1).min(1.0),
    cpu_power: cpu_w,
    gpu_power: gpu_w,
    ane_power: ane_w,
    all_power: all,
    sys_power: all + 9.0 + 2.0 * noise(t + 5),
    ..Default::default()
  }
}

/// (name, user, base CPU %, memory MiB, base W, GPU %)
const PROCS: &[(&str, &str, f32, u64, f32, f32)] = &[
  ("cargo", "dev", 0.0, 210, 0.0, 0.0),
  ("rustc", "dev", 0.0, 1650, 0.0, 0.0),
  ("WindowServer", "_windowserver", 18.0, 430, 0.0, 9.0),
  ("Google Chrome Helper (GPU)", "dev", 9.6, 418, 0.26, 7.7),
  ("Google Chrome Helper (Renderer)", "dev", 8.4, 525, 0.17, 0.0),
  ("Code Helper (Renderer)", "dev", 8.1, 739, 0.15, 1.0),
  ("rust-analyzer", "dev", 6.8, 1434, 0.14, 0.0),
  ("Google Chrome", "dev", 4.8, 612, 0.12, 0.0),
  ("ghostty", "dev", 3.4, 312, 0.07, 1.8),
  ("Code", "dev", 3.4, 523, 0.08, 0.0),
  ("Spotify", "dev", 1.8, 338, 0.04, 0.0),
  ("Slack", "dev", 1.5, 383, 0.03, 0.0),
  ("mds_stores", "root", 2.6, 64, 0.0, 0.0),
  ("Code Helper (Plugin)", "dev", 2.2, 307, 0.04, 0.0),
  ("Slack Helper (Renderer)", "dev", 1.4, 290, 0.02, 0.0),
  ("kernel_task", "root", 4.1, 12, 0.0, 0.0),
  ("Finder", "dev", 0.4, 96, 0.01, 0.0),
  ("Mail", "dev", 0.3, 188, 0.01, 0.0),
  ("logd", "root", 0.9, 93, 0.0, 0.0),
  ("macmon", "dev", 1.1, 18, 0.03, 0.0),
  ("coreaudiod", "_coreaudiod", 0.8, 21, 0.0, 0.0),
  ("Dock", "dev", 0.2, 74, 0.0, 0.0),
  ("Notes", "dev", 0.1, 140, 0.0, 0.0),
  ("syspolicyd", "root", 0.6, 114, 0.0, 0.0),
  ("Messages", "dev", 0.1, 122, 0.0, 0.0),
  ("Calendar", "dev", 0.0, 98, 0.0, 0.0),
  ("Activity Monitor", "dev", 0.6, 61, 0.01, 0.0),
  ("cloudd", "dev", 0.2, 45, 0.0, 0.0),
];

pub(super) fn procs(t: u32) -> Vec<ProcInfo> {
  let build = load(t, 2, 0.12);
  PROCS
    .iter()
    .enumerate()
    .map(|(i, &(name, user, cpu, mib, watts, gpu))| {
      let i32_ = i as u32;
      let jitter = 0.7 + 0.6 * noise(t * 17 + i32_);
      let (cpu, watts) = match name {
        "rustc" => (720.0 * build, 9.5 * build),
        "cargo" => (35.0 * build, 0.4 * build),
        _ => (cpu * jitter, watts * jitter),
      };
      let own = user == "dev";
      ProcInfo {
        pid: 300 + (i32_ * 2711 % 60000) as i32,
        name: name.to_string(),
        path: format!("/Applications/{name}"),
        user: user.to_string(),
        cpu_pct: cpu,
        cpu_time_ns: (u64::from(i32_) * 7919 + 600) * 1_000_000_000 % (90_000 * 1_000_000_000),
        mem_bytes: mib << 20,
        power_w: own.then_some(watts),
        gpu_pct: gpu * jitter,
        started: own.then_some((1000 + u64::from(i32_), 0)),
      }
    })
    .collect()
}

/// A process sampler for `run_procs_thread`.
pub(super) fn proc_sampler() -> impl FnMut() -> Vec<ProcInfo> {
  let mut t = HISTORY;
  move || {
    t += 1;
    procs(t)
  }
}

/// Process actions that touch nothing: demo PIDs are made up and may belong to real processes.
/// Every demo process reads as alive, a signal "succeeds" without being sent.
#[derive(Debug)]
pub(super) struct DemoSys;

impl ProcSys for DemoSys {
  fn signal(&self, _pid: i32, _sig: i32) -> Result<(), i32> {
    Ok(())
  }

  fn identity(&self, pid: i32) -> Option<(u64, u64)> {
    procs(HISTORY).into_iter().find(|p| p.pid == pid).and_then(|p| p.started)
  }

  fn open_files(&self, _pid: i32) -> Result<Vec<OpenFile>, i32> {
    let file = |fd, kind, text: &str, mode| OpenFile { fd, kind, text: text.to_string(), mode };
    Ok(vec![
      file(None, Kind::Cwd, "/Users/dev/src/app", ""),
      file(Some(0), Kind::Dev, "/dev/null", "r-"),
      file(Some(1), Kind::Dev, "/dev/ttys004", "rw"),
      file(Some(2), Kind::Dev, "/dev/ttys004", "rw"),
      file(Some(3), Kind::File, "/Users/dev/Library/Caches/app/index.db", "rw"),
      file(Some(4), Kind::File, "/Users/dev/Library/Caches/app/index.db-wal", "rw"),
      file(Some(5), Kind::Dir, "/Users/dev/src/app/target", "r-"),
      file(Some(6), Kind::Tcp, "127.0.0.1:52144 → 127.0.0.1:9090 ESTABLISHED", ""),
      file(Some(7), Kind::Tcp, "*:8080 LISTEN", ""),
      file(Some(8), Kind::Udp, "*:5353", ""),
      file(Some(9), Kind::Unix, "/var/run/mDNSResponder", ""),
      file(Some(10), Kind::Pipe, "", ""),
      file(Some(11), Kind::Pipe, "", ""),
      file(Some(12), Kind::Kqueue, "", ""),
    ])
  }

  fn reveal(&self, _path: &str) {}

  fn copy(&self, _text: &str) {}
}
