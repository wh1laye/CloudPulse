use serde::Serialize;
use sysinfo::{System, Disks, Networks};
use std::process::Command;
use std::fs;
use std::path::Path;

#[derive(Serialize)]
struct ProcessInfo {
    name: String,
    cpu: f32,
}

#[derive(Serialize, Clone)]
struct GpuInfo {
    name: String,
    usage: f32,
    memory_used: u64,
    memory_total: u64,
    temperature: u32,
    vendor: String,
}

#[derive(Serialize)]
struct SystemStats {
    cpu_usage: f32,
    cpu_name: String,
    cpu_cores: usize,
    ram_used: u64,
    ram_total: u64,
    disk_used: u64,
    disk_total: u64,
    network_received: u64,
    network_transmitted: u64,
    hostname: String,
    os: String,
    kernel: String,
    arch: String,
    uptime: u64,
    processes_count: usize,
    top_processes: Vec<ProcessInfo>,
    gpus: Vec<GpuInfo>,
}

fn get_nvidia_gpu_info() -> Option<GpuInfo> {
    let output = Command::new("nvidia-smi")
        .args(&["--query-gpu=name,utilization.gpu,memory.used,memory.total,temperature.gpu", "--format=csv,noheader,nounits"])
        .output()
        .ok()?;
    
    if !output.status.success() {
        return None;
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let first_gpu = stdout.lines().next()?;
    let parts: Vec<&str> = first_gpu.split(',').map(|s| s.trim()).collect();
    
    if parts.len() >= 5 {
        Some(GpuInfo {
            name: parts[0].to_string(),
            usage: parts[1].parse().unwrap_or(0.0),
            memory_used: parts[2].parse::<u64>().unwrap_or(0) * 1024 * 1024,
            memory_total: parts[3].parse::<u64>().unwrap_or(0) * 1024 * 1024,
            temperature: parts[4].parse().unwrap_or(0),
            vendor: "NVIDIA".to_string(),
        })
    } else {
        None
    }
}

fn get_amd_gpu_info() -> Option<GpuInfo> {
    // Ищем AMD GPU в sysfs
    let drm_path = Path::new("/sys/class/drm");
    
    if !drm_path.exists() {
        return None;
    }
    
    // Ищем card0, card1, ... с device/vendor_id = AMD (0x1002)
    for entry in fs::read_dir(drm_path).ok()? {
        let entry = entry.ok()?;
        let path = entry.path();
        
        // Ищем card0, card1, но НЕ card0-HDMI и подобные
        let name = path.file_name()?.to_str()?;
        if !name.starts_with("card") || name.contains('-') {
            continue;
        }
        
        let device_path = path.join("device");
        if !device_path.exists() {
            continue;
        }
        
        // Проверяем vendor_id
        let vendor_id = fs::read_to_string(device_path.join("vendor"))
            .ok()?
            .trim()
            .to_lowercase();
        
        if !vendor_id.contains("1002") {
            // Не AMD
            continue;
        }
        
        // Нашли AMD GPU!
        
        // Название устройства (через lspci)
        let gpu_name = get_gpu_name_from_lspci(&device_path);
        
        // GPU загрузка (gpu_busy_percent)
        let usage = fs::read_to_string(device_path.join("gpu_busy_percent"))
            .ok()
            .and_then(|s| s.trim().parse::<f32>().ok())
            .unwrap_or(0.0);
        
        // VRAM used / total
        let memory_used = fs::read_to_string(device_path.join("mem_info_vram_used"))
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(0);
        
        let memory_total = fs::read_to_string(device_path.join("mem_info_vram_total"))
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(0);
        
        // Температура (через hwmon)
        let temperature = get_gpu_temperature(&device_path);
        
        return Some(GpuInfo {
            name: gpu_name,
            usage,
            memory_used,
            memory_total,
            temperature,
            vendor: "AMD".to_string(),
        });
    }
    
    None
}

fn get_gpu_name_from_lspci(device_path: &Path) -> String {
    // Получаем PCI адрес из device_path
    // device_path выглядит как /sys/class/drm/card0/device
    // реальный путь: /sys/devices/pci0000:00/0000:00:01.0/0000:01:00.0/...
    
    // Попробуем прочитать uevent
    if let Ok(uevent) = fs::read_to_string(device_path.join("uevent")) {
        for line in uevent.lines() {
            if line.starts_with("PCI_SLOT_NAME=") {
                let pci_slot = line.trim_start_matches("PCI_SLOT_NAME=");
                // Вызовем lspci
                if let Ok(output) = Command::new("lspci")
                    .args(&["-s", pci_slot, "-v"])
                    .output()
                {
                    if output.status.success() {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        if let Some(first_line) = stdout.lines().next() {
                            // Формат: "01:00.0 VGA compatible controller: Advanced Micro Devices..."
                            if let Some(colon_pos) = first_line.find(':') {
                                if let Some(colon_pos2) = first_line[colon_pos + 1..].find(':') {
                                    let name = first_line[colon_pos + colon_pos2 + 2..].trim();
                                    return name.to_string();
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    "AMD Radeon GPU".to_string()
}

fn get_gpu_temperature(device_path: &Path) -> u32 {
    // Ищем hwmon в device/hwmon/hwmonX/
    let hwmon_path = device_path.join("hwmon");
    
    if let Ok(entries) = fs::read_dir(&hwmon_path) {
        for entry in entries.flatten() {
            let temp_path = entry.path().join("temp1_input");
            if let Ok(temp_str) = fs::read_to_string(temp_path) {
                // Температура в millidegrees (1000 = 1°C)
                if let Ok(temp) = temp_str.trim().parse::<u32>() {
                    return temp / 1000;
                }
            }
        }
    }
    
    0
}

fn get_gpu_info() -> Vec<GpuInfo> {
    let mut gpus = Vec::new();
    
    // Попробуем NVIDIA
    if let Some(nvidia) = get_nvidia_gpu_info() {
        gpus.push(nvidia);
        return gpus;
    }
    
    // Попробуем AMD через sysfs (без rocm-smi!)
    if let Some(amd) = get_amd_gpu_info() {
        gpus.push(amd);
        return gpus;
    }
    
    // Попробуем rocm-smi как fallback
    if let Some(amd) = get_amd_gpu_info_rocm() {
        gpus.push(amd);
    }
    
    gpus
}

fn get_amd_gpu_info_rocm() -> Option<GpuInfo> {
    let output = Command::new("rocm-smi")
        .args(&["--showuse", "--showmeminfo", "vram", "--showtemp", "--csv"])
        .output()
        .ok()?;
    
    if !output.status.success() {
        return None;
    }
    
    Some(GpuInfo {
        name: "AMD GPU".to_string(),
        usage: 0.0,
        memory_used: 0,
        memory_total: 0,
        temperature: 0,
        vendor: "AMD".to_string(),
    })
}

#[tauri::command]
fn get_system_stats() -> SystemStats {
    let mut sys = System::new_all();
    sys.refresh_all();
    
    std::thread::sleep(std::time::Duration::from_millis(100));
    sys.refresh_cpu_all();
    
    let cpus = sys.cpus();
    let cpu_usage = if cpus.is_empty() {
        0.0
    } else {
        cpus.iter().map(|c| c.cpu_usage()).sum::<f32>() / cpus.len() as f32
    };
    
    let cpu_name = cpus.first()
        .map(|c| c.brand().to_string())
        .unwrap_or_else(|| "Unknown CPU".to_string());
    let cpu_cores = cpus.len();
    
    let ram_used = sys.used_memory();
    let ram_total = sys.total_memory();
    
    let disks = Disks::new_with_refreshed_list();
    let main_disk = disks.list().first();
    let (disk_used, disk_total) = match main_disk {
        Some(disk) => (
            disk.total_space() - disk.available_space(),
            disk.total_space(),
        ),
        None => (0, 0),
    };
    
    let networks = Networks::new_with_refreshed_list();
    let mut total_received = 0u64;
    let mut total_transmitted = 0u64;
    for (_, data) in &networks {
        total_received += data.total_received();
        total_transmitted += data.total_transmitted();
    }
    
    let hostname = System::host_name().unwrap_or_else(|| "unknown".to_string());
    let os = System::long_os_version().unwrap_or_else(|| "unknown".to_string());
    let kernel = System::kernel_version().unwrap_or_else(|| "unknown".to_string());
    let arch = std::env::consts::ARCH.to_string();
    let uptime = System::uptime();
    
    let mut processes: Vec<_> = sys.processes().values().collect();
    processes.sort_by(|a, b| b.cpu_usage().partial_cmp(&a.cpu_usage()).unwrap());
    
    let top_processes: Vec<ProcessInfo> = processes
        .iter()
        .take(5)
        .map(|p| ProcessInfo {
            name: p.name().to_string_lossy().to_string(),
            cpu: p.cpu_usage(),
        })
        .collect();
    
    let gpus = get_gpu_info();
    
    SystemStats {
        cpu_usage,
        cpu_name,
        cpu_cores,
        ram_used,
        ram_total,
        disk_used,
        disk_total,
        network_received: total_received,
        network_transmitted: total_transmitted,
        hostname,
        os,
        kernel,
        arch,
        uptime,
        processes_count: sys.processes().len(),
        top_processes,
        gpus,
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![get_system_stats])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
